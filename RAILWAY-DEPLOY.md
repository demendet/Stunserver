# Railway Deployment Instructions - FIXED

## 🚀 Quick Deploy

```bash
# 1. Install Railway CLI  
npm install -g @railway/cli

# 2. Login to Railway
railway login

# 3. Deploy the signaling server
cd /mnt/c/Users/mattd/Desktop/FlightSimP2P
railway up
```

## 📋 What Gets Deployed

**✅ Rust Signaling Server** - Deployed to Railway
- Handles WebRTC handshake (offers/answers/ICE)
- Ultra-fast Rust WebSocket server
- Uses Docker build from root Dockerfile

**🌐 STUN Servers** - Use Google's free STUN servers
- No deployment needed (Railway doesn't support UDP well)
- Google STUN servers work great for NAT traversal
- Multiple servers for redundancy

## 🔧 Environment Variables

Railway auto-sets:
- `PORT=3000` 
- `RUST_LOG=info`

## 🌐 Getting Your URL

After deployment:
- **Signaling**: `wss://your-app-name.railway.app`

## 🎯 Update C# Client

In your AvaloniaTest:
```csharp
p2pService.SignalingServerUrl = "wss://your-app-name.railway.app";
// STUN servers are pre-configured to use Google's free servers
```

## ✅ Architecture

```
Your Apps ←→ Direct P2P WebRTC ←→ Your Apps
    ↓                              ↓
    └→ Railway Signaling Server ←──┘ (handshake only)
    └→ Google STUN Servers ←───────┘ (NAT traversal)
```

Perfect for ultra-low latency flight sim data!