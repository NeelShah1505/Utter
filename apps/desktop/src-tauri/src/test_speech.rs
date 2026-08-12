use objc2::rc::Id;
use objc2::runtime::Object;
use objc2::{class, msg_send, msg_send_id};

pub fn request_speech_permission() {
    unsafe {
        let cls = class!(SFSpeechRecognizer);
        // SFSpeechRecognizer is in Speech framework, we must link it.
        // We'll just do a quick test if it compiles.
    }
}
