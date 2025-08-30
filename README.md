# FlightSim P2P WebRTC Solution

Ultra-low latency P2P connection system for flight simulator data sync using WebRTC and STUN.

## Architecture

```
Client A ←→ Direct P2P WebRTC Data Channel ←→ Client B
    ↓                                           ↓
    └→ Signaling Server (handshake only) ←─────┘
    └→ STUN Server (NAT traversal only) ←──────┘
```

## Key Features

- **Direct P2P**: Data flows directly between clients (no server hops)
- **Ultra-low latency**: Eliminates SignalR server bottleneck  
- **NAT traversal**: Works through firewalls without port forwarding
- **Secure**: DTLS encryption on data channels
- **Railway deployable**: Docker containerized

## Components

1. **STUN Server** (`stun/`): coturn server for NAT traversal
2. **Signaling Server** (`signaling/`): WebSocket server for WebRTC handshake
3. **Client Integration** (`client/`): WebRTC data channel implementation
4. **Docker Setup**: Railway deployment configuration

## Quick Start

```bash
# Start the stack
docker-compose up

# Deploy to Railway
railway up
```

The current SignalR flow: `Sim → App → SignalR Server → Client` becomes `Sim → App ←P2P→ Client` facks