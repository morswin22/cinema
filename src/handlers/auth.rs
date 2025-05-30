use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
    Form,
    http::StatusCode,
};
use r2d2::PooledConnection;
use tower_sessions::Session;
use axum::extract::Extension;
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::{prelude::*, r2d2::ConnectionManager};
use askama::Template;
use htmxtools::response::HxRedirect;
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
        register_error: Option<diesel::result::Error>
    }

    let template = Tmpl {
        register_error: None
    };
    Ok(Html(template.render()?))
}

pub async fn handle_register(
    State(pool): State<Arc<MysqlPool>>,
    Form(form): Form<RegisterForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    
    let hashed_password = hash(form.password, DEFAULT_COST).unwrap();
    let new_user = NewUser {
        email: &form.email,
        password: &hashed_password,
    };

    let result = diesel::insert_into(users)
        .values(&new_user)
        .execute(&mut conn);

    match result {
        Ok(_) => Ok(HxRedirect::from(Uri::from_static("/login")).into_response()),
        Err(error) => {
            #[derive(Debug, Template)]
            #[template(path = "register_form.html")]
            struct Tmpl {
                register_error: Option<diesel::result::Error>
            }

            let template = Tmpl {
                register_error: Some(error)
            };
            Ok(Html(template.render()?).into_response())
        }
    }
}

pub async fn show_login() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "login.html")]
    struct Tmpl {
        login_error: Option<AppError>
    }

    let template = Tmpl {
        login_error: None
    };
    Ok(Html(template.render()?))
}

pub async fn handle_login(
    State(pool): State<Arc<MysqlPool>>,
    session: Session,
    Form(form): Form<LoginForm>,
) -> Result<Response, AppError> {
    let mut conn = pool.get().map_err(|e| AppError::PoolError(e.to_string()))?;
    
    let result = (|| {
        let user = users
            .filter(email.eq(&form.email))
            .first::<User>(&mut conn).map_err(|_| AppError::UserLoginError)?;

        if !verify(form.password, &user.password).unwrap_or(false) {
            return Err(AppError::UserLoginError);
        }

        Ok(user)
    })();

    if let Ok(user) = result { 
        session.insert(SESSION_USER_KEY, user).await.unwrap();
        return Ok(HxRedirect::from(Uri::from_static("/")).into_response());
    } else {
        #[derive(Debug, Template)]
        #[template(path = "login_form.html")]
        struct Tmpl {
            login_error: Option<AppError>
        }

        let template = Tmpl {
            login_error: Some(AppError::UserLoginError)
        };
        Ok(Html(template.render()?).into_response())
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.unwrap();
    Redirect::to("/")
}
