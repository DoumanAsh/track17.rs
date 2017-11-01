extern crate serde;
extern crate serde_json;

mod hyper {
    extern crate hyper;
    extern crate hyper_tls;

    pub use self::hyper::error::Error;
    pub use self::hyper::{Client, Request, Method, Response, StatusCode, Chunk};
    pub use self::hyper::header::{ContentType, ContentLength, Referer, Origin, UserAgent, Accept};
    pub use self::hyper::client::{HttpConnector, FutureResponse};
    pub use self::hyper_tls::{HttpsConnector};

    header! { (XRequestedWith, "X-Requested-With") => [String] }
}

use ::tokio_core::reactor::{
    Handle
};

const TRACKER_URL: &'static str = "https://www.17track.net/restapi/handlertrack.ashx";

pub mod payload {
    use super::serde;
    use super::serde::Deserialize;

    use std::fmt;

    #[derive(Serialize, Debug)]
    pub struct RequestData {
        pub num: String
    }

    #[derive(Serialize, Debug)]
    pub struct Request {
        guid: String,
        data: Vec<RequestData>
    }

    impl Request {
        pub fn simple(num: String) -> Self {
            Request {
                guid: "".to_string(),
                data: vec![RequestData {
                    num
                }]
            }
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct TrackEvent {
        #[serde(rename = "a")]
        date: String,
        #[serde(rename = "c")]
        location: String,
        #[serde(rename = "z")]
        message: String,
    }

    impl fmt::Display for TrackEvent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}    {}, {}", self.date, self.location, self.message)
        }
    }

    ///Tracking data.
    #[derive(Deserialize, Debug)]
    pub struct TrackData {
        ///Last event, which is of most interest
        #[serde(rename = "z0")]
        pub last: TrackEvent,
        ///All other events, which seems to start from the beginning.
        #[serde(rename = "z1")]
        pub all: Vec<TrackEvent>
    }

    impl fmt::Display for TrackData {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Last:\n{}\n", self.last)?;
            for event in self.all.iter() {
                write!(f, "----------\n{}\n", event)?;
            }

            Ok(())
        }
    }

    #[derive(Deserialize, Debug)]
    ///Represents tracking data on 1 parcel.
    pub struct ResponsData {
        #[serde(rename = "no")]
        pub num: String,
        ///Delay, presumebly, if >0 means try again later
        pub delay: usize,
        ///If delay is >0 then track data is null
        pub track: Option<TrackData>
    }

    pub fn response_result_de<'de, D: serde::de::Deserializer<'de>>(result: D) -> Result<Result<(), String>, D::Error> {
        let result = String::deserialize(result)?;

        match result.as_ref() {
            "Ok" => Ok(Ok(())),
            _ => Ok(Err(result))
        }
    }

    #[derive(Deserialize, Debug)]
    pub struct Response {
        #[serde(rename = "msg")]
        #[serde(deserialize_with="response_result_de")]
        pub result: Result<(), String>,
        #[serde(rename = "dat")]
        pub data: Vec<ResponsData>
    }
}

pub struct Client {
    hyper: hyper::Client<hyper::HttpsConnector<hyper::HttpConnector>>,
}

impl Client {
    pub fn new(handle: Handle) -> Self {
        let hyper = hyper::Client::configure().keep_alive(true)
                                              .connector(hyper::HttpsConnector::new(4, &handle).unwrap())
                                              .build(&handle);

        Client {
            hyper,
        }
    }

    ///Requests tracking information.
    ///
    ///Due to no official API the request is performed by emulating browser's request.
    ///Which of course means it is likely to break, if they change internals.
    pub fn track(&self, num: &str) -> hyper::FutureResponse {
        let mut req = hyper::Request::new(hyper::Method::Post, TRACKER_URL.parse().unwrap());
        let payload = payload::Request::simple(num.to_string());

        req.headers_mut().set(hyper::ContentType::json());
        req.headers_mut().set(hyper::Origin::new("https", "www.17track.net", None));
        req.headers_mut().set(hyper::Referer::new(format!("http://www.17track.net/pt/track?nums={}&fc=0", num)));
        req.headers_mut().set(hyper::UserAgent::new("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/49.0.2623.87 Safari/537.36"));
        req.headers_mut().set(hyper::XRequestedWith("XMLHttpRequest".to_string()));
        req.headers_mut().set(hyper::Accept::star());
        req.headers_mut().set(hyper::ContentType::form_url_encoded());
        req.set_body(serde_json::to_string(&payload).unwrap());

        self.hyper.request(req)
    }

    pub fn parse_track_response(body: hyper::Chunk) -> Result<payload::Response, serde_json::Error> {
        serde_json::from_slice(&body)
    }
}
