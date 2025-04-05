use libmq::{nt_channel, pack::MessagePackPacker};

pub mod call;
pub mod response;

use call::Call;
use response::Response;

nt_channel!(
    SampleClient,
    SampleServer,
    Call,
    Response,
    MessagePackPacker
);
