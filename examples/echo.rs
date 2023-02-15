use wasi::wasi_io::{
    read, subscribe, subscribe_read, write, InputStream, OutputStream, StreamError,
};
use wasi::wasi_poll::{Pollable, poll_oneoff};

#[allow(dead_code)] // TODO
fn echo_server(streams: &[(InputStream, OutputStream)]) -> Result<(), StreamError> {
    let input_pollables = streams.iter()
        .map(|(input, _output)| subscribe_read(*input))
        .collect::<Vec<Pollable>>();

    loop {
        let readies = poll_oneoff(&input_pollables);

        for ((input, output), ready) in streams.iter().zip(readies.iter()) {
            if *ready == 0 {
                continue;
            }

            let (data, end_of_stream) = read(*input, usize::MAX as u64)?;

            let mut view = &data[..];
            while !view.is_empty() {
                let pollable = subscribe(*output);
                poll_oneoff(&[pollable]);

                let nwritten = write(*output, view)?;
                view = &view[nwritten as usize..];
            }

            if end_of_stream {
                return Ok(());
            }
        }
    }
}

fn main() {
    // TODO: obtain some sockets and call echo_server on them.
}
