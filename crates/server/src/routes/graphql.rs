use crate::auth::AuthenticatedUser;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use miniflux_client::schema::AppSchema;

pub async fn graphql_handler(
    State(schema): State<AppSchema>,
    _user: AuthenticatedUser,
    req: GraphQLRequest,
) -> Response {
    let enabled = std::env::var("GRAPHQL_ENABLED")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);
    if !enabled {
        return (StatusCode::NOT_FOUND, "GraphQL endpoint is disabled").into_response();
    }
    GraphQLResponse::from(schema.execute(req.into_inner()).await).into_response()
}
