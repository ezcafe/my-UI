use crate::auth::AuthenticatedUser;
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::State;
use miniflux_client::schema::AppSchema;

pub async fn graphql_handler(
    State(schema): State<AppSchema>,
    _user: AuthenticatedUser,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
