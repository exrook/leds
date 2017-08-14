//use futures::{Future, Stream, Sink};
//use futures::future;
//use futures::sync::BiLock;
//
//trait MyEncode {
//    type EncodeInput;
//    type EncodeOutput;
//    fn encode(&mut self, Self::EncodeInput) -> Self::EncodeOutput;
//}
//
//trait MyDecode {
//    type DecodeInput;
//    type DecodeOutput;
//    fn decode(&mut self, Self::DecodeInput) -> Self::DecodeOutput;
//}
//
//trait MyFramed: MyEncode + MyDecode {}
//
//fn wrap<S, T>(
//    s: S,
//    framed: T,
//) -> impl Stream<Item = T::DecodeOutput, Error = S::Error>
//         + Sink<SinkItem = T::EncodeInput, SinkError = S::SinkError>
//where
//    S: Stream + Sink,
//    T: MyFramed<DecodeInput = S::Item, EncodeOutput = S::SinkItem>,
//{
//    let (a, b) = BiLock::new(framed);
//    let mut b = Some(b);
//    s.and_then(move |x| {
//        let a = a;
//        future::poll_fn(|| Ok(a.poll_lock()))
//        //a.take()
//        //    .unwrap()
//        //    .lock()
//            .map(|mut framed| {
//                framed.decode(x)
//            })
//    }).with(move |x| {
//            b.take()
//                .unwrap()
//                .lock()
//                .map(|mut framed| framed.encode(x))
//                .map_err(|_| unreachable!())
//        })
//}
