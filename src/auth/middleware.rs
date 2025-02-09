use poem::{http::{self, StatusCode}, Endpoint, Error, Middleware, Request, Result};
use poem_grants::authorities::AttachAuthorities;

use super::jwt;

#[derive(Clone)]
pub struct JwtMiddleware {
    pub manager: jwt::Manager
}

impl<E: Endpoint> Middleware<E> for JwtMiddleware {
    type Output = JwtMiddlewareImpl<E>;

    fn transform(&self, ep: E) -> Self::Output {
        JwtMiddlewareImpl { ep, manager: self.manager.clone() }
    }
}

pub struct JwtMiddlewareImpl<E> {
    ep: E,
    manager: jwt::Manager
}

impl<E: Endpoint> Endpoint for JwtMiddlewareImpl<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> Result<Self::Output> {
        if let Some(value) = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.starts_with("Bearer "))
            .map(|value| &value[7..])
        {
            let jwt_data = self.manager.decode(value)?;

            if jwt_data.is_expired() {
                return Err(Error::from_status(StatusCode::UNAUTHORIZED))
            }

            req.attach(jwt_data.permissions);
        }

        self.ep.call(req).await
    }
}