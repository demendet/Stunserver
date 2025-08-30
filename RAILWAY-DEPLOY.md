# Railway Deployment Instructions

## 🚀 Quick Deploy

```bash
# 1. Install Railway CLI
npm install -g @railway/cli

# 2. Login to Railway  
railway login

# 3. Initialize project
cd /mnt/c/Users/mattd/Desktop/FlightSimP2P
railway init

# 4. Deploy both services
railway up
```

## 📋 Manual Railway Setup

1. Go to [railway.app](https://railway.app)
2. Create new project
3. Connect your GitHub repo
4. Railway will auto-detect the `railway.toml` config
5. Deploy both services automatically

## 🔧 Environment Variables

Railway will automatically set:
- `PORT=3000` for signaling server
- `RUST_LOG=info` for logging

## 🌐 Getting URLs

After deployment, Railway gives you:
- **Signaling Server**: `https://your-app-signaling.railway.app`
- **STUN Server**: `stun://your-app-stun.railway.app:3478`

## 🎯 Update C# Client

In your AvaloniaTest, update the server URLs:
```csharp
p2pService.SignalingServerUrl = "wss://your-app-signaling.railway.app";
p2pService.StunServers = new[] { "stun:your-app-stun.railway.app:3478" };
```

## ✅ Expected Results

- **Signaling Server**: Handles WebRTC handshake only
- **STUN Server**: NAT traversal for P2P connections  
- **Your Apps**: Direct P2P ultra-low latency communication

The servers only handle initial connection setup - all flight sim data flows directly P2P!