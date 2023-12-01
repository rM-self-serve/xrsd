use hex::FromHexError;
use std::env;
use std::io::{self, Read, Write};

const LINEBUFFSIZE: usize = 30;
const WRITEBUFFSIZE: usize = LINEBUFFSIZE * 68;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        cli_info();
        return Ok(());
    }

    let query = &args[1];

    if query == "-f" {
        return forward();
    }

    if query == "-r" {
        return reverse();
    }

    cli_info();

    Ok(())
}

fn forward() -> io::Result<()> {
    const HOLDBUFFSIZE: usize = (LINEBUFFSIZE * 2) + 1;

    let mut wbuffv = Vec::with_capacity(WRITEBUFFSIZE);

    let sinlock = io::stdin().lock();
    let mut soutlock = io::stdout().lock();

    let mut buff = [0 as u8; LINEBUFFSIZE];
    let mut hold_buff = [0 as u8; HOLDBUFFSIZE];
    hold_buff[LINEBUFFSIZE * 2] = b'\n';
    let mut ndx = 0;

    for (i, byte) in sinlock.bytes().enumerate() {
        buff[i % LINEBUFFSIZE] = byte.unwrap();

        if i % LINEBUFFSIZE == LINEBUFFSIZE - 1 {
            myencode_to_slice(&buff, &mut hold_buff).unwrap();
            wbuffv.push(hold_buff);

            if ndx % WRITEBUFFSIZE == WRITEBUFFSIZE - 1 {
                let df: Vec<u8> = wbuffv.as_slice().into_iter().flat_map(|val| *val).collect();
                soutlock.write_all(&df).unwrap();
                wbuffv.clear();
            }
        }

        ndx += 1;
    }

    let mut df: Vec<u8> = wbuffv.as_slice().into_iter().flat_map(|val| *val).collect();

    let remain = ndx % LINEBUFFSIZE;
    if remain > 0 {
        let h = hex::encode(&buff[..remain]) + "\n";
        df.append(&mut h.as_bytes().to_vec());
    }

    soutlock.write_all(&df).unwrap();
    return Ok(());
}

fn reverse() -> io::Result<()> {
    let sin = io::stdin();
    let mut soutlock = io::stdout().lock();

    let mut out_vec = Vec::with_capacity(WRITEBUFFSIZE);

    for line in sin.lines() {
        let l = line.unwrap();
        let mut out = hex::decode(&l).unwrap();
        out_vec.append(&mut out);
        if out_vec.len() == WRITEBUFFSIZE {
            soutlock.write_all(&out_vec).unwrap();
            out_vec.clear();
        }
    }
    soutlock.write_all(&out_vec).unwrap();
    return Ok(());
}

fn cli_info() {
    println!("xrsd\n");
    println!("A small utility to convert data to hex and back.");
    println!("Only works on stdin and stdout. Equivalent to 'xxd -p'.");
    println!("https://github.com/rM-self-serve/xrsd\n");
    println!("-f   Encode data into hex");
    println!("-r   Decode hex into data\n");
}

fn myencode_to_slice<T: AsRef<[u8]>>(input: T, output: &mut [u8]) -> Result<(), FromHexError> {
    let inp_ref = input.as_ref();
    let inp_len = inp_ref.len();

    for (byte, (i, j)) in inp_ref.iter().zip(generate_iter(inp_len * 2)) {
        let (high, low) = byte2hex(*byte);
        output[i] = high;
        output[j] = low;
    }

    Ok(())
}

fn generate_iter(len: usize) -> impl Iterator<Item = (usize, usize)> {
    (0..len).step_by(2).zip((0..len).skip(1).step_by(2))
}

const HEX_CHARS_LOWER: &[u8; 16] = b"0123456789abcdef";
fn byte2hex(byte: u8) -> (u8, u8) {
    let high = HEX_CHARS_LOWER[((byte & 0xf0) >> 4) as usize];
    let low = HEX_CHARS_LOWER[(byte & 0x0f) as usize];

    (high, low)
}
