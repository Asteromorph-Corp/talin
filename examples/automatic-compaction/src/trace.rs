use std::io;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talin::chain::OutputCtor;
use talin::strategy::Strategy;
use serde::Serialize;

use crate::evidence::Evidence;
use crate::report::{Report, RequestEvidence, ResponseEvidence};

#[derive(Clone)]
pub struct Trace {
    pub report: Arc<Mutex<Report>>,
    pub noise_marker: String,
    pub request_budget: usize,
}

#[async_trait]
impl<L: Evidence, O: OutputCtor> Strategy<L, O> for Trace
where
    L::Request: Serialize,
{
    type Output = ();
    type Error = io::Error;

    async fn process_request(&mut self, request: L::Request) -> Result<L::Request, io::Error> {
        let encoded = serde_json::to_string(&request)?;
        let evidence = RequestEvidence {
            bytes: encoded.len(),
            contains_archive_noise: encoded.contains(&self.noise_marker),
            compactions: L::request_compactions(&request),
        };
        let mut report = self.report.lock().unwrap();
        if report.requests.len() >= self.request_budget {
            return Err(io::Error::other("live request budget exhausted"));
        }
        report.requests.push(evidence);
        eprintln!(
            "API request {}: {} bytes",
            report.requests.len(),
            encoded.len()
        );
        Ok(request)
    }

    async fn process_response(&mut self, response: L::Response) -> Result<L::Response, io::Error> {
        let evidence = ResponseEvidence {
            compactions: L::response_compactions(&response),
            usage: L::response_usage(&response),
        };
        eprintln!(
            "API response: {} compaction block(s)",
            evidence.compactions.len()
        );
        self.report.lock().unwrap().responses.push(evidence);
        Ok(response)
    }

    async fn finalize(self) -> Result<(), io::Error> {
        Ok(())
    }
}
