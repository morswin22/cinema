use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
};
use tower_sessions::Session;
use axum::extract::Extension;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::prelude::*;
use askama::Template;
use htmxtools::response::HxPushUrl;
use axum::http::Uri;
use std::sync::Arc;

use crate::{
    forms::auth::{LoginForm, RegisterForm},
    models::{NewUser, User},
    schema::users::dsl::*,
    AppError, MysqlPool, SESSION_USER_KEY
};

pub async fn show_register() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "register.html")]
    struct Tmpl {
    }

    let template = Tmpl {
    };
    Ok(Html(template.render()?))
}

pub async fn handle_register(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<RegisterForm>,
) -> Result<impl IntoResponse, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    
    let hashed_password = hash(form.password, DEFAULT_COST).unwrap();
    let new_user = NewUser {
        email: &form.email,
        password: &hashed_password,
    };

    diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut conn).unwrap();

    // TODO: handle register errors

    Ok(HxPushUrl::url(Uri::from_static("/login")))
}

pub async fn show_login() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "login.html")]
    struct Tmpl {
    }

    let template = Tmpl {
    };
    Ok(Html(template.render()?))
}

pub async fn handle_login(
    State(pool): State<Arc<MysqlPool>>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    
    let user = users
        .filter(email.eq(&form.email))
        .first::<User>(&mut conn).unwrap();

    if verify(form.password, &user.password).unwrap() {
        session.insert(SESSION_USER_KEY, user).await.unwrap();
        return Ok(HxPushUrl::url(Uri::from_static("/")).into_response());
    } else {
        #[derive(Debug, Template)]
        #[template(path = "login_form.html")]
        struct Tmpl {
        }

        // TODO: handle login errors

        let template = Tmpl {
        };
        Ok(Html(template.render()?).into_response())
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.unwrap();
    Redirect::to("/")
}
