use quinn::{RecvStream, SendStream};

use crate::{
    constant::MAXIMUM_MESSAGE_LENGTH, error::Error, message::Message, request::Request,
    response::Response,
};

pub const AUTHENTICATION_MAX_READ_LENGHT: usize = 64;
pub const REQUEST_MAX_READ_LENGHT: usize = MAXIMUM_MESSAGE_LENGTH * 2;
pub const RESPONCE_MAX_READ_LENGHT: usize = MAXIMUM_MESSAGE_LENGTH * 2;

pub struct Network;

impl Network {
    pub async fn send_response(
        response: &Response,
        mut send_stream: SendStream,
    ) -> Result<(), Error> {
        Ok({
            send_stream.write_all(&bitcode::encode(response)).await?;
            send_stream.finish()?;
        })
    }

    pub async fn receive_request(
        mut receive_stream: RecvStream,
        size_limit: Option<usize>,
    ) -> Result<Request, Error> {
        let request = match size_limit {
            Some(size_limit) => receive_stream.read_to_end(size_limit).await?,
            None => receive_stream.read_to_end(REQUEST_MAX_READ_LENGHT).await?,
        };

        Ok(bitcode::decode::<Request>(&request)?)
    }

    pub async fn send_request_and_receive_response(
        request: &Request,
        mut send_stream: SendStream,
        mut receive_stream: RecvStream,
    ) -> Result<Response, Error> {
        send_stream.write_all(&bitcode::encode(request)).await?;
        send_stream.finish()?;

        let response = receive_stream.read_to_end(RESPONCE_MAX_READ_LENGHT).await?;

        Ok(bitcode::decode::<Response>(&response)?)
    }

    pub async fn send_message(
        message: &Message,
        send_stream: &mut SendStream,
    ) -> Result<(), Error> {
        let payload = bitcode::encode(message);
        send_stream
            .write_all(&(payload.len() as u32).to_le_bytes())
            .await?;
        send_stream.write_all(&payload).await?;

        Ok(())
    }

    pub async fn receive_message(receive_stream: &mut RecvStream) -> Result<Message, Error> {
        let mut length_bytes = [0u8; 4];
        receive_stream.read_exact(&mut length_bytes).await?;

        let length = u32::from_le_bytes(length_bytes) as usize;
        if length > RESPONCE_MAX_READ_LENGHT {
            return Err(crate::error::network::Error::ReadBoundExceed(
                length,
                RESPONCE_MAX_READ_LENGHT,
            )
            .into());
        }

        let mut payload = vec![0u8; length];
        receive_stream.read_exact(&mut payload).await?;

        Ok(bitcode::decode(&payload)?)
    }
}
