# AvaloniaTest P2P Integration Guide

## Replace SignalR with WebRTC P2P

### Step 1: Add NuGet Packages to AvaloniaTest.csproj
```xml
<PackageReference Include="Microsoft.MixedReality.WebRTC" Version="2.0.2" />
<PackageReference Include="WebSocketSharp" Version="1.0.3-netstandard2.0" />
```

### Step 2: Copy WebRTCP2PService.cs
Copy `WebRTCP2PService.cs` to your `Services/` folder in AvaloniaTest.

### Step 3: Update MainWindow.axaml.cs

Replace SharedCockpitService initialization:
```csharp
// OLD - remove this
// sharedCockpitService = new SharedCockpitService();

// NEW - add this
private WebRTCP2PService p2pService;

private void InitializeComponents()
{
    // existing code...
    
    // replace signalr with p2p
    p2pService = new WebRTCP2PService();
    p2pService.SignalingServerUrl = "wss://your-app.railway.app"; // update with your railway url
    
    // wire up events (same as before)
    p2pService.Connected += OnSessionConnected;
    p2pService.Disconnected += OnSessionDisconnected;  
    p2pService.PilotFlyingChanged += OnPilotFlyingChanged;
    p2pService.ClientConnected += OnClientConnected;
    p2pService.ClientDisconnected += OnClientDisconnected;
    p2pService.AircraftDataReceived += OnAircraftDataReceived;
    p2pService.LogMessage += (s, msg) => LogSharedCockpitMessage($"[P2P] {msg}");
}
```

### Step 4: Update Connection Methods

Replace SignalR connection calls:
```csharp
// OLD method calls - replace these
// await sharedCockpitService.ConnectAsHost(sessionCode, clientId);
// await sharedCockpitService.ConnectAsClient(sessionCode, clientId);
// await sharedCockpitService.SendAircraftData(compressedData);

// NEW P2P method calls
await p2pService.ConnectAsHost(sessionCode, clientId);
await p2pService.ConnectAsClient(sessionCode, clientId); 
await p2pService.SendAircraftData(compressedData);
```

### Step 5: Update UI Connection Logic

In your button click handlers:
```csharp
private async void HostButton_Click(object sender, RoutedEventArgs e)
{
    var sessionCode = GenerateSessionCode(); // your existing method
    var clientId = "YourClientName";
    
    var success = await p2pService.ConnectAsHost(sessionCode, clientId);
    if (success)
    {
        LogSharedCockpitMessage($"🚀 Hosting P2P session: {sessionCode}");
        // update UI
    }
}

private async void JoinButton_Click(object sender, RoutedEventArgs e)
{
    var sessionCode = sessionCodeTextBox.Text;
    var clientId = "YourClientName";
    
    var success = await p2pService.ConnectAsClient(sessionCode, clientId);
    if (success)
    {
        LogSharedCockpitMessage($"🤝 Joined P2P session: {sessionCode}");
        // update UI  
    }
}
```

### Step 6: Test the Integration

1. Deploy the Rust servers to Railway
2. Update the SignalingServerUrl in your code
3. Test P2P connection between two AvaloniaTest instances
4. Verify direct data transfer (check logs for "P2P connection established!")

## Expected Performance Improvements

- **Latency**: ~1-5ms (was ~50-100ms with SignalR)
- **Responsiveness**: Instant control sync
- **Bandwidth**: Much more efficient 
- **Server Load**: Near zero after handshake

## Troubleshooting

### Connection Issues
- Check firewall allows WebRTC traffic
- Verify STUN server is accessible
- Check Railway deployment logs

### NAT Traversal Problems  
- Try different STUN servers
- Enable TURN fallback for strict NATs
- Check router UPnP settings

### Performance Issues
- Monitor P2P establishment success rate
- Check for packet loss
- Verify data channel is actually being used (not falling back to relay)