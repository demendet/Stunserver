.PHONY: build start stop deploy clean test

# ultra-fast flightsim p2p system

build:
	@echo "🦀 Building optimized Rust signaling server..."
	cd signaling && cargo build --release
	@echo "🐳 Building Docker containers..."
	docker-compose build

start:
	@echo "🚀 Starting FlightSim P2P system..."
	docker-compose up -d
	@echo "✅ System running at:"
	@echo "   Signaling: http://localhost:3000"
	@echo "   STUN: stun://localhost:3478"

stop:
	@echo "🛑 Stopping P2P system..."
	docker-compose down

deploy:
	@echo "🚢 Deploying to Railway..."
	railway up

test:
	@echo "🧪 Testing Rust signaling server..."
	cd signaling && cargo test
	@echo "🔍 Testing containers..."
	docker-compose up --abort-on-container-exit

clean:
	@echo "🧹 Cleaning up..."
	docker-compose down -v
	docker system prune -f
	cd signaling && cargo clean

logs:
	@echo "📝 Viewing logs..."
	docker-compose logs -f

dev:
	@echo "🛠️ Development mode..."
	cd signaling && cargo run

# quick start for development
quick: build start logs