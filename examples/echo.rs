use wasi::poll::{drop_pollable, poll_oneoff, Pollable};
use wasi::streams::{
    read, subscribe_to_input_stream, subscribe_to_output_stream, write, InputStream, OutputStream,
    StreamError,
};

#[allow(dead_code)] // TODO
fn echo_server(streams: &[(InputStream, OutputStream)]) -> Result<(), StreamError> {
    let input_pollables = streams
        .iter()
        .map(|(input, _output)| subscribe_to_input_stream(*input))
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
                let pollable = subscribe_to_output_stream(*output);
                poll_oneoff(&[pollable]);
                drop_pollable(pollable);

                let nwritten = write(*output, view)?;
                view = &view[nwritten as usize..];
            }

            if end_of_stream {
                for pollable in input_pollables {
                    drop_pollable(pollable);
                }
                return Ok(());
            }
        }
    }
}

fn main() {
    // TODO: obtain some sockets and call echo_server on them.
}
