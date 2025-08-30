# WebRTC C# Integration for AvaloniaTest

## NuGet Package Required
```xml
<PackageReference Include="Microsoft.MixedReality.WebRTC" Version="2.0.2" />
<PackageReference Include="System.Text.Json" Version="8.0.0" />
<PackageReference Include="WebSocketSharp" Version="1.0.3-netstandard2.0" />
```

## Integration Steps

1. **Add WebRTCP2PService.cs** to your Services folder
2. **Replace SharedCockpitService** calls with WebRTCP2PService calls
3. **Update connection flow** in MainWindow.axaml.cs

## Key Benefits vs SignalR

- **Direct P2P**: Data flows directly between apps (0 server hops)
- **Ultra-low latency**: ~1-5ms vs 50-100ms with SignalR  
- **Bandwidth efficient**: No server relay overhead
- **Secure**: DTLS encryption on data channels
- **Firewall friendly**: Works through NAT without port forwarding

## Connection Flow

1. Both apps connect to lightweight signaling server
2. Exchange WebRTC handshake (SDP offers/answers)  
3. Use STUN for NAT traversal
4. Establish direct P2P data channel
5. **All flight sim data flows directly P2P**

## Usage

```csharp
// Create P2P service
var p2pService = new WebRTCP2PService();

// Host creates session
await p2pService.CreateSessionAsync();
var sessionCode = p2pService.SessionCode;

// Client joins session  
await p2pService.JoinSessionAsync(sessionCode);

// Send flight data directly P2P
await p2pService.SendFlightDataAsync(aircraftData);
```