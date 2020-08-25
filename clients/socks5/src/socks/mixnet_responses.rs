use super::client::ActiveStreams;
use client_core::client::received_buffer::ReconstructedMessagesReceiver;
use client_core::client::received_buffer::{ReceivedBufferMessage, ReceivedBufferRequestSender};
use futures::channel::mpsc;
use futures::StreamExt;
use log::*;
use nymsphinx::receiver::ReconstructedMessage;
use ordered_buffer::OrderedMessageBuffer;
use socks5_requests::Response;

pub(crate) struct MixnetResponseListener {
    buffer_requester: ReceivedBufferRequestSender,
    mix_response_receiver: ReconstructedMessagesReceiver,
    active_streams: ActiveStreams,
    response_buffer: OrderedMessageBuffer,
}

impl Drop for MixnetResponseListener {
    fn drop(&mut self) {
        self.buffer_requester
            .unbounded_send(ReceivedBufferMessage::ReceiverDisconnect)
            .expect("the buffer request failed!")
    }
}

impl MixnetResponseListener {
    pub(crate) fn new(
        buffer_requester: ReceivedBufferRequestSender,
        active_streams: ActiveStreams,
    ) -> Self {
        let (mix_response_sender, mix_response_receiver) = mpsc::unbounded();
        buffer_requester
            .unbounded_send(ReceivedBufferMessage::ReceiverAnnounce(mix_response_sender))
            .unwrap();

        let response_buffer = OrderedMessageBuffer::new();
        MixnetResponseListener {
            active_streams,
            buffer_requester,
            mix_response_receiver,
            response_buffer,
        }
    }

    async fn on_message(&mut self, reconstructed_message: ReconstructedMessage) {
        let raw_message = reconstructed_message.message;
        if reconstructed_message.reply_SURB.is_some() {
            println!("this message had a surb - we didn't do anything with it");
        }
        let response = match Response::try_from_bytes(&raw_message) {
            Err(err) => {
                warn!("failed to parse received response - {:?}", err);
                return;
            }
            Ok(data) => {
                self.response_buffer.write(data.message.clone());
                data
            }
        };

        let mut active_streams_guard = self.active_streams.lock().await;

        // `remove` gives back the entry (assuming it exists). There's no reason
        // for it to persist after we send data back
        if let Some(stream_receiver) = active_streams_guard.remove(&response.connection_id) {
            if let Some(msg) = self.response_buffer.read() {
                stream_receiver.send(msg).unwrap()
            }
        } else {
            warn!(
                "no connection_id exists with id: {:?}",
                &response.connection_id
            )
        }
    }

    pub(crate) async fn run(&mut self) {
        while let Some(received_responses) = self.mix_response_receiver.next().await {
            for reconstructed_message in received_responses {
                self.on_message(reconstructed_message).await;
            }
        }
        println!("We should never see this message");
    }
}
