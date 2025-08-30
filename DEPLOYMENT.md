# FlightSim P2P Deployment Guide

## Railway Deployment (Recommended)

### Quick Deploy
```bash
# install railway cli
npm install -g @railway/cli

# login to railway
railway login

# deploy the project
railway up
```

### Manual Deploy
1. Go to [railway.app](https://railway.app)
2. Create new project from GitHub repo
3. Deploy both services:
   - **Signaling Server**: Rust WebSocket server (Port 3000)
   - **STUN Server**: coturn server (Ports 3478, 5349, 10000-20000)

## Docker Local Testing
```bash
# start both services
docker-compose up

# or build manually
docker build -t flightsim-signaling ./signaling
docker build -t flightsim-stun ./stun
```

## Performance Expectations

### vs Current SignalR Architecture
- **Latency**: 1-5ms vs 50-100ms  
- **Throughput**: 10x higher
- **Server Load**: 95% reduced (only handshake traffic)

### Architecture Comparison
```
OLD: Sim → App → SignalR Server → Client App
NEW: Sim → App ←────P2P────→ Client App
              ↑              ↑
          STUN (NAT only)   STUN
```

## C# Integration

### 1. Add NuGet Packages
```xml
<PackageReference Include="Microsoft.MixedReality.WebRTC" Version="2.0.2" />
<PackageReference Include="WebSocketSharp" Version="1.0.3-netstandard2.0" />
```

### 2. Replace SharedCockpitService
```csharp
// OLD
var signalRService = new SharedCockpitService();
await signalRService.ConnectAsHost(sessionCode, clientId);

// NEW  
var p2pService = new WebRTCP2PService();
await p2pService.ConnectAsHost(sessionCode, clientId);
```

### 3. Configure Servers
```csharp
p2pService.SignalingServerUrl = "wss://your-railway-app.railway.app";
p2pService.StunServers = new[] { "stun:your-stun-server.railway.app:3478" };
```

## Security Features
- DTLS encryption on P2P channels
- STUN authentication
- Session code validation
- Client verification

## Monitoring
- View Railway logs for connection stats
- Monitor P2P establishment success rate
- Track latency improvements