//! An owned public-transport reservation in the same request-wide response allowance.
//! Unused capacity is refunded; actual serialization remains cumulatively charged.
use super::{Cost, Request, Stop};
pub(crate) struct ResponseCredit {
    request: Request,
    capacity: u64,
}
impl Request {
    pub(crate) fn response_capacity(&self, capacity: u64) -> Result<ResponseCredit, Stop> {
        let _permit = self.reserve_nested(
            Cost {
                response_bytes: capacity,
                ..Cost::default()
            },
            true,
        )?;
        {
            let mut s = self.inner.state.lock().expect("response capacity");
            s.response_reserved = s
                .response_reserved
                .checked_add(capacity)
                .expect("admitted response capacity");
        }
        Ok(ResponseCredit {
            request: self.clone(),
            capacity,
        })
    }
}
impl ResponseCredit {
    pub(crate) fn commit(mut self, actual: u64) -> Result<(), Stop> {
        if actual > self.capacity {
            return Err(self.request.stop(Stop::ResponseBytes));
        }
        let unused = self.capacity - actual;
        {
            let mut s = self.request.inner.state.lock().expect("response credit");
            s.used.response_bytes -= unused;
            s.response_reserved -= self.capacity;
            s.finalization_used.response_bytes -= unused;
        }
        self.capacity = 0;
        Ok(())
    }
}
impl Drop for ResponseCredit {
    fn drop(&mut self) {
        let mut s = self.request.inner.state.lock().expect("response credit");
        s.used.response_bytes -= self.capacity;
        s.response_reserved -= self.capacity;
        s.finalization_used.response_bytes -= self.capacity;
    }
}
