use anyhow::Result;
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::interval;
use tokio_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use tracing::{error, info, warn};
use uuid::Uuid;

type ClientId = String;
type SessionCode = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SignalMessage {
    #[serde(rename = "create-session")]
    CreateSession,
    #[serde(rename = "join-session")]
    JoinSession { sessionCode: String },
    #[serde(rename = "webrtc-offer")]
    WebRTCOffer { sdp: String },
    #[serde(rename = "webrtc-answer")]
    WebRTCAnswer { sdp: String },
    #[serde(rename = "ice-candidate")]
    IceCandidate { candidate: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ResponseMessage {
    #[serde(rename = "connected")]
    Connected { clientId: String },
    #[serde(rename = "session-created")]
    SessionCreated { sessionCode: String, role: String },
    #[serde(rename = "session-joined")]
    SessionJoined {
        sessionCode: String,
        role: String,
        hostId: String,
    },
    #[serde(rename = "client-joined")]
    ClientJoined { clientId: String },
    #[serde(rename = "peer-disconnected")]
    PeerDisconnected { clientId: String },
    #[serde(rename = "webrtc-offer")]
    WebRTCOffer { sdp: String, from: String },
    #[serde(rename = "webrtc-answer")]
    WebRTCAnswer { sdp: String, from: String },
    #[serde(rename = "ice-candidate")]
    IceCandidate { candidate: String, from: String },
    #[serde(rename = "error")]
    Error { message: String },
}

#[derive(Debug)]
pub struct Client {
    id: ClientId,
    tx: tokio::sync::mpsc::UnboundedSender<Message>,
    session_code: Option<SessionCode>,
}

#[derive(Debug)]
pub struct Session {
    code: SessionCode,
    host_id: ClientId,
    clients: Vec<ClientId>,
    created_at: u64,
}

#[derive(Clone)]
pub struct AppState {
    clients: Arc<DashMap<ClientId, Client>>,
    sessions: Arc<DashMap<SessionCode, Session>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("flightsim_p2p_signaling=info")
        .init();

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    let state = AppState::new();
    
    // spawn cleanup task
    let cleanup_state = state.clone();
    tokio::spawn(cleanup_old_sessions(cleanup_state));

    info!("🚀 FlightSim P2P Signaling Server running on {}", addr);

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(handle_connection(state, stream, addr));
    }

    Ok(())
}

async fn handle_connection(state: AppState, stream: TcpStream, addr: SocketAddr) {
    match accept_async(stream).await {
        Ok(ws_stream) => {
            let client_id = Uuid::new_v4().to_string();
            info!("✅ Client {} connected from {}", client_id, addr);
            
            if let Err(e) = handle_client(state, ws_stream, client_id.clone()).await {
                error!("❌ Error handling client {}: {}", client_id, e);
            }
        }
        Err(e) => {
            error!("❌ WebSocket connection failed from {}: {}", addr, e);
        }
    }
}

async fn handle_client(
    state: AppState,
    ws_stream: WebSocketStream<TcpStream>,
    client_id: ClientId,
) -> Result<()> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    // register client
    let client = Client {
        id: client_id.clone(),
        tx,
        session_code: None,
    };
    state.clients.insert(client_id.clone(), client);

    // send connected message
    let msg = ResponseMessage::Connected {
        clientId: client_id.clone(),
    };
    let _ = ws_sender.send(Message::Text(serde_json::to_string(&msg)?)).await;

    // spawn sender task
    let sender_client_id = client_id.clone();
    let sender_state = state.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
        // cleanup on sender task end
        cleanup_client(&sender_state, &sender_client_id).await;
    });

    // handle incoming messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = handle_message(&state, &client_id, &text).await {
                    warn!("⚠️ Error handling message from {}: {}", client_id, e);
                }
            }
            Ok(Message::Close(_)) => break,
            Ok(_) => {} // ignore other message types
            Err(e) => {
                warn!("⚠️ WebSocket error for client {}: {}", client_id, e);
                break;
            }
        }
    }

    cleanup_client(&state, &client_id).await;
    Ok(())
}

async fn handle_message(state: &AppState, client_id: &str, text: &str) -> Result<()> {
    let message: SignalMessage = serde_json::from_str(text)?;

    match message {
        SignalMessage::CreateSession => {
            handle_create_session(state, client_id).await?;
        }
        SignalMessage::JoinSession { sessionCode } => {
            handle_join_session(state, client_id, &sessionCode).await?;
        }
        SignalMessage::WebRTCOffer { sdp } => {
            handle_webrtc_signaling(state, client_id, ResponseMessage::WebRTCOffer {
                sdp,
                from: client_id.to_string(),
            }).await?;
        }
        SignalMessage::WebRTCAnswer { sdp } => {
            handle_webrtc_signaling(state, client_id, ResponseMessage::WebRTCAnswer {
                sdp,
                from: client_id.to_string(),
            }).await?;
        }
        SignalMessage::IceCandidate { candidate } => {
            handle_webrtc_signaling(state, client_id, ResponseMessage::IceCandidate {
                candidate,
                from: client_id.to_string(),
            }).await?;
        }
    }

    Ok(())
}

async fn handle_create_session(state: &AppState, client_id: &str) -> Result<()> {
    let session_code = generate_session_code();
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let session = Session {
        code: session_code.clone(),
        host_id: client_id.to_string(),
        clients: vec![client_id.to_string()],
        created_at: now,
    };

    state.sessions.insert(session_code.clone(), session);

    // update client session
    if let Some(mut client) = state.clients.get_mut(client_id) {
        client.session_code = Some(session_code.clone());
    }

    send_to_client(state, client_id, ResponseMessage::SessionCreated {
        sessionCode: session_code.clone(),
        role: "host".to_string(),
    }).await?;

    info!("🎯 Session {} created by {}", session_code, client_id);
    Ok(())
}

async fn handle_join_session(state: &AppState, client_id: &str, session_code: &str) -> Result<()> {
    let mut session = state.sessions.get_mut(session_code)
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    if session.clients.len() >= 2 {
        send_to_client(state, client_id, ResponseMessage::Error {
            message: "Session full".to_string(),
        }).await?;
        return Ok(());
    }

    let host_id = session.host_id.clone();
    session.clients.push(client_id.to_string());

    // update client session
    if let Some(mut client) = state.clients.get_mut(client_id) {
        client.session_code = Some(session_code.to_string());
    }

    send_to_client(state, client_id, ResponseMessage::SessionJoined {
        sessionCode: session_code.to_string(),
        role: "client".to_string(),
        hostId: host_id.clone(),
    }).await?;

    // notify host
    send_to_client(state, &host_id, ResponseMessage::ClientJoined {
        clientId: client_id.to_string(),
    }).await?;

    info!("🤝 Client {} joined session {}", client_id, session_code);
    Ok(())
}

async fn handle_webrtc_signaling(state: &AppState, client_id: &str, message: ResponseMessage) -> Result<()> {
    let client = state.clients.get(client_id)
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    let session_code = client.session_code.clone()
        .ok_or_else(|| anyhow::anyhow!("Client not in session"))?;

    let session = state.sessions.get(&session_code)
        .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

    // find the other client in the session
    for other_client_id in &session.clients {
        if other_client_id != client_id {
            send_to_client(state, other_client_id, message).await?;
            break;
        }
    }

    Ok(())
}

async fn send_to_client(state: &AppState, client_id: &str, message: ResponseMessage) -> Result<()> {
    if let Some(client) = state.clients.get(client_id) {
        let msg_text = serde_json::to_string(&message)?;
        let _ = client.tx.send(Message::Text(msg_text));
    }
    Ok(())
}

async fn cleanup_client(state: &AppState, client_id: &str) {
    let session_code = if let Some(client) = state.clients.get(client_id) {
        client.session_code.clone()
    } else {
        None
    };

    // remove client
    state.clients.remove(client_id);

    // handle session cleanup
    if let Some(session_code) = session_code {
        if let Some(mut session) = state.sessions.get_mut(&session_code) {
            session.clients.retain(|id| id != client_id);

            // notify other clients
            for other_client_id in &session.clients {
                let _ = send_to_client(state, other_client_id, ResponseMessage::PeerDisconnected {
                    clientId: client_id.to_string(),
                }).await;
            }

            // remove empty session
            if session.clients.is_empty() {
                drop(session);
                state.sessions.remove(&session_code);
                info!("🗑️ Session {} deleted", session_code);
            }
        }
    }

    info!("👋 Client {} disconnected", client_id);
}

async fn cleanup_old_sessions(state: AppState) {
    let mut interval = interval(Duration::from_secs(1800)); // 30 minutes
    
    loop {
        interval.tick().await;
        
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let max_age = 1800; // 30 minutes

        state.sessions.retain(|code, session| {
            let keep = session.clients.is_empty() && (now - session.created_at) < max_age;
            if !keep {
                info!("🧹 Cleaned up old session {}", code);
            }
            keep
        });
    }
}

fn generate_session_code() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| rng.gen_range(0..36))
        .map(|n| if n < 10 { 
            (b'0' + n) as char 
        } else { 
            (b'A' + n - 10) as char 
        })
        .collect()
}