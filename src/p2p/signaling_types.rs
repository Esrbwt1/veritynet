// src/p2p/signaling_types.rs
use serde::{Deserialize, Serialize};
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;

// Represents all possible messages sent over the WebSocket signaling channel
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")] // Adds a "type" field to JSON for easy matching
pub enum SignalingMessage {
    Offer(SdpData),
    Answer(SdpData),
    IceCandidate(IceCandidateData),
    Error(String), // To report errors via signaling
                   // Add more message types later if needed (e.g., Hello, Identify)
}

// Contains SDP data (used for Offer and Answer)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SdpData {
    pub sdp: String,
    // Optionally add target peer ID if needed for signaling server logic
    // pub target_peer_id: Option<String>,
}

// Contains ICE Candidate data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IceCandidateData {
    pub candidate: String,
    #[serde(rename = "sdpMid")] // Match common JS naming conventions
    pub sdp_mid: Option<String>,
    #[serde(rename = "sdpMLineIndex")] // Match common JS naming conventions
    pub sdp_mline_index: Option<u16>,
    // Optionally add target peer ID
    // pub target_peer_id: Option<String>,
}

// Helper function to create SdpData from RTCSessionDescription
impl From<RTCSessionDescription> for SdpData {
    fn from(desc: RTCSessionDescription) -> Self {
        SdpData { sdp: desc.sdp }
    }
}

// Helper function to create IceCandidateData from RTCIceCandidateInit
// Note: RTCIceCandidateInit is not Serialize/Deserialize directly
// We convert it to our serializable format.
impl From<RTCIceCandidateInit> for IceCandidateData {
    fn from(candidate_init: RTCIceCandidateInit) -> Self {
        IceCandidateData {
            candidate: candidate_init.candidate,
            sdp_mid: candidate_init.sdp_mid,
            sdp_mline_index: candidate_init.sdp_mline_index,
        }
    }
}

// Helper to potentially convert back (may need error handling)
impl TryFrom<IceCandidateData> for RTCIceCandidateInit {
    type Error = String; // Simple error type for now

    fn try_from(data: IceCandidateData) -> Result<Self, Self::Error> {
        Ok(RTCIceCandidateInit {
            candidate: data.candidate,
            sdp_mid: data.sdp_mid,
            sdp_mline_index: data.sdp_mline_index,
            username_fragment: None, // Not typically sent over signaling
        })
    }
}