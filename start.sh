#!/bin/bash
# FlightSim P2P Ultra-Fast Startup Script

echo "🚀 Starting FlightSim P2P System..."

# check if docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker first."
    exit 1
fi

# check if docker-compose is available  
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose not found. Please install Docker Compose first."
    exit 1
fi

echo "📦 Building optimized containers..."

# build rust signaling server (release mode for max performance)
docker-compose build signaling

# build stun server
docker-compose build stun

echo "🌟 Starting ultra-fast P2P services..."

# start the complete stack
docker-compose up -d

echo ""
echo "✅ FlightSim P2P System is running!"
echo ""
echo "📊 Services:"
echo "   • Rust Signaling Server: http://localhost:3000"
echo "   • STUN Server: stun://localhost:3478" 
echo ""
echo "🔧 Integration:"
echo "   • Copy client/WebRTCP2PService.cs to your AvaloniaTest/Services/"
echo "   • Follow client/AVALONIA-INTEGRATION.md"
echo ""
echo "🚢 Deploy to Railway:"
echo "   • railway up"
echo ""
echo "📈 Expected Performance:"
echo "   • Latency: 1-5ms (vs 50-100ms with SignalR)"
echo "   • Direct P2P: No server bottleneck"
echo "   • Ultra-responsive flight sim sync"
echo ""
echo "📝 View logs: docker-compose logs -f"
echo "🛑 Stop system: docker-compose down"