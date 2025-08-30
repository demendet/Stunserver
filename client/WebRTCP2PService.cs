using System;
using System.Collections.Generic;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using Microsoft.MixedReality.WebRTC;
using WebSocketSharp;

namespace AvaloniaTest.Services
{
    // ultra-fast p2p webrtc service - direct connection no server hops
    public class WebRTCP2PService : IDisposable
    {
        private WebSocket _signalingSocket;
        private PeerConnection _peerConnection;
        private DataChannel _dataChannel;
        
        private string _clientId;
        private string _sessionCode;
        private bool _isHost = false;
        private bool _isConnected = false;
        private bool _p2pEstablished = false;
        
        // events for ui integration
        public event EventHandler Connected;
        public event EventHandler Disconnected; 
        public event EventHandler<string> PilotFlyingChanged;
        public event EventHandler<string> ClientConnected;
        public event EventHandler<string> ClientDisconnected;
        public event EventHandler<byte[]> AircraftDataReceived;
        public event EventHandler<string> LogMessage;
        
        // server urls (configurable)
        public string SignalingServerUrl { get; set; } = "ws://localhost:3000";
        public List<string> StunServers { get; set; } = new() { "stun:stun.l.google.com:19302" };
        
        public bool IsConnected => _isConnected && _p2pEstablished;
        public string SessionCode => _sessionCode;
        public string ClientId => _clientId;
        public bool IsHost => _isHost;
        
        public async Task<bool> ConnectAsHost(string sessionCode, string clientId)
        {
            _clientId = clientId;
            _isHost = true;
            
            try 
            {
                await InitializeWebRTC();
                await ConnectToSignalingServer();
                await CreateSession();
                return true;
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke(this, $"Host connection failed: {ex.Message}");
                return false;
            }
        }
        
        public async Task<bool> ConnectAsClient(string sessionCode, string clientId)
        {
            _sessionCode = sessionCode;
            _clientId = clientId;
            _isHost = false;
            
            try
            {
                await InitializeWebRTC();
                await ConnectToSignalingServer();
                await JoinSession();
                return true;
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke(this, $"Client connection failed: {ex.Message}");
                return false;
            }
        }
        
        private async Task InitializeWebRTC()
        {
            // create peer connection with stun servers
            var config = new PeerConnectionConfiguration
            {
                IceServers = StunServers.Select(url => new IceServer { Urls = { url } }).ToList()
            };
            
            _peerConnection = new PeerConnection();
            await _peerConnection.InitializeAsync(config);
            
            // ice candidate handling
            _peerConnection.LocalSdpReadytoSend += OnLocalSdpReady;
            _peerConnection.IceCandidateReadytoSend += OnIceCandidateReady;
            _peerConnection.Connected += OnPeerConnected;
            _peerConnection.DataChannelAdded += OnDataChannelAdded;
            
            LogMessage?.Invoke(this, "WebRTC initialized");
        }
        
        private async Task ConnectToSignalingServer()
        {
            _signalingSocket = new WebSocket(SignalingServerUrl);
            
            _signalingSocket.OnMessage += OnSignalingMessage;
            _signalingSocket.OnError += (s, e) => LogMessage?.Invoke(this, $"Signaling error: {e.Message}");
            _signalingSocket.OnClose += (s, e) => LogMessage?.Invoke(this, "Signaling disconnected");
            
            _signalingSocket.Connect();
            await Task.Delay(1000); // wait for connection
            
            LogMessage?.Invoke(this, "Connected to signaling server");
        }
        
        private async Task CreateSession()
        {
            // create data channel for flight data
            _dataChannel = await _peerConnection.AddDataChannelAsync("flightdata", true, true);
            _dataChannel.MessageReceived += OnDataChannelMessage;
            
            var message = new { type = "create-session" };
            _signalingSocket.Send(JsonSerializer.Serialize(message));
        }
        
        private async Task JoinSession()
        {
            var message = new { type = "join-session", sessionCode = _sessionCode };
            _signalingSocket.Send(JsonSerializer.Serialize(message));
        }
        
        private async void OnSignalingMessage(object sender, MessageEventArgs e)
        {
            try
            {
                var message = JsonSerializer.Deserialize<JsonElement>(e.Data);
                var messageType = message.GetProperty("type").GetString();
                
                switch (messageType)
                {
                    case "connected":
                        _clientId = message.GetProperty("clientId").GetString();
                        break;
                        
                    case "session-created":
                        _sessionCode = message.GetProperty("sessionCode").GetString();
                        _isConnected = true;
                        LogMessage?.Invoke(this, $"Session created: {_sessionCode}");
                        Connected?.Invoke(this, EventArgs.Empty);
                        break;
                        
                    case "session-joined":
                        _isConnected = true;
                        LogMessage?.Invoke(this, $"Joined session: {_sessionCode}");
                        Connected?.Invoke(this, EventArgs.Empty);
                        break;
                        
                    case "client-joined":
                        var joinedClientId = message.GetProperty("clientId").GetString();
                        LogMessage?.Invoke(this, $"Client joined: {joinedClientId}");
                        // host creates offer
                        if (_isHost)
                        {
                            await CreateOffer();
                        }
                        ClientConnected?.Invoke(this, joinedClientId);
                        break;
                        
                    case "webrtc-offer":
                        var offerSdp = message.GetProperty("sdp").GetString();
                        await HandleOffer(offerSdp);
                        break;
                        
                    case "webrtc-answer":
                        var answerSdp = message.GetProperty("sdp").GetString();
                        await HandleAnswer(answerSdp);
                        break;
                        
                    case "ice-candidate":
                        var candidate = message.GetProperty("candidate").GetString();
                        await HandleIceCandidate(candidate);
                        break;
                        
                    case "peer-disconnected":
                        var disconnectedId = message.GetProperty("clientId").GetString();
                        ClientDisconnected?.Invoke(this, disconnectedId);
                        _p2pEstablished = false;
                        break;
                        
                    case "error":
                        var error = message.GetProperty("message").GetString();
                        LogMessage?.Invoke(this, $"Signaling error: {error}");
                        break;
                }
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke(this, $"Message handling error: {ex.Message}");
            }
        }
        
        private async Task CreateOffer()
        {
            var offer = await _peerConnection.CreateOffer();
            await _peerConnection.SetLocalDescription(offer);
        }
        
        private async Task HandleOffer(string sdpString)
        {
            var offer = SdpMessage.CreateOffer(sdpString);
            await _peerConnection.SetRemoteDescription(offer);
            
            var answer = await _peerConnection.CreateAnswer();
            await _peerConnection.SetLocalDescription(answer);
        }
        
        private async Task HandleAnswer(string sdpString)
        {
            var answer = SdpMessage.CreateAnswer(sdpString);
            await _peerConnection.SetRemoteDescription(answer);
        }
        
        private async Task HandleIceCandidate(string candidateString)
        {
            var parts = candidateString.Split('|');
            if (parts.Length >= 3)
            {
                var candidate = new IceCandidate 
                {
                    Content = parts[0],
                    SdpMlineIndex = int.Parse(parts[1]),
                    SdpMid = parts[2]
                };
                _peerConnection.AddIceCandidate(candidate);
            }
        }
        
        private void OnLocalSdpReady(SdpMessage message)
        {
            var messageType = message.Type == SdpMessageType.Offer ? "webrtc-offer" : "webrtc-answer";
            var signalMessage = new 
            { 
                type = messageType, 
                sdp = message.Content 
            };
            _signalingSocket.Send(JsonSerializer.Serialize(signalMessage));
        }
        
        private void OnIceCandidateReady(IceCandidate candidate)
        {
            var candidateString = $"{candidate.Content}|{candidate.SdpMlineIndex}|{candidate.SdpMid}";
            var message = new 
            { 
                type = "ice-candidate", 
                candidate = candidateString 
            };
            _signalingSocket.Send(JsonSerializer.Serialize(message));
        }
        
        private void OnPeerConnected()
        {
            _p2pEstablished = true;
            LogMessage?.Invoke(this, "🚀 P2P connection established! Direct flight data transfer active");
        }
        
        private void OnDataChannelAdded(DataChannel channel)
        {
            if (channel.Label == "flightdata")
            {
                _dataChannel = channel;
                _dataChannel.MessageReceived += OnDataChannelMessage;
                LogMessage?.Invoke(this, "Data channel connected");
            }
        }
        
        private void OnDataChannelMessage(byte[] data)
        {
            // direct p2p flight data received
            AircraftDataReceived?.Invoke(this, data);
        }
        
        public async Task SendAircraftData(byte[] data)
        {
            if (_p2pEstablished && _dataChannel != null && _dataChannel.State == DataChannelState.Open)
            {
                try
                {
                    _dataChannel.SendMessage(data);
                }
                catch (Exception ex)
                {
                    LogMessage?.Invoke(this, $"Send failed: {ex.Message}");
                }
            }
        }
        
        public async Task TakeControl()
        {
            // implement control takeover if needed
            PilotFlyingChanged?.Invoke(this, _clientId);
        }
        
        public async Task GiveControl(string toClientId)
        {
            // implement control transfer if needed  
            PilotFlyingChanged?.Invoke(this, toClientId);
        }
        
        public async Task Disconnect()
        {
            try
            {
                _p2pEstablished = false;
                _isConnected = false;
                
                _dataChannel?.Dispose();
                _peerConnection?.Dispose();
                _signalingSocket?.Close();
                
                LogMessage?.Invoke(this, "Disconnected");
                Disconnected?.Invoke(this, EventArgs.Empty);
            }
            catch (Exception ex)
            {
                LogMessage?.Invoke(this, $"Disconnect error: {ex.Message}");
            }
        }
        
        public void Dispose()
        {
            _ = Disconnect();
        }
    }
}