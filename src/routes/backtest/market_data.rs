use std::{path::PathBuf, sync::Arc};

use actix_web::{error::ErrorInternalServerError, web, Error, HttpResponse};

use crate::{
    app_state::AppState,
    data_handlers::{
        pipeline,
        utils::{datetime_str_to_i64, i64_to_datetime_str},
    },
    data_models::market_data::{
        kline::KLine,
        market_data::{
            GetMarketDataRequest, MarketDataDatesRequest, MarketDataDatesResponse, MarketDataFront,
        },
    },
    db_handlers::market_data::{
        get_db_market_data_dates, get_market_data_page, insert_market_data,
    },
};

pub async fn downloaded_market_data(
    data: web::Data<AppState>,
    r: web::Query<GetMarketDataRequest>,
) -> Result<HttpResponse, Error> {
    let market_data = get_downloaded_market_data(&data, r.into_inner()).await?;

    Ok(HttpResponse::Ok().json(market_data))
}

pub async fn get_downloaded_market_data(
    data: &web::Data<AppState>,
    r: GetMarketDataRequest,
) -> Result<Vec<MarketDataFront>, Error> {
    let data = Arc::clone(data);

    let market_data = get_market_data_page(&data.pool, &r)
        .await
        .map_err(ErrorInternalServerError)?;

    Ok(market_data)
}

pub async fn download_market_data(
    data: web::Data<AppState>,
    r: web::Json<MarketDataFront>,
) -> Result<HttpResponse, Error> {
    let result = _download_market_data(data, r).await?;

    Ok(HttpResponse::Ok().json(result))
}

pub async fn _download_market_data(
    data: web::Data<AppState>,
    r: web::Json<MarketDataFront>,
) -> Result<MarketDataFront, Error> {
    let data_path = PathBuf::from(data.app_settings.data_path.clone());

    pipeline::pipeline::<KLine>(
        data_path.clone(),
        data.app_settings.binance_data_url.clone(),
        r.exchange.to_lowercase(),
        r.symbol.to_lowercase(),
        r.market_data_type.clone(),
        datetime_str_to_i64(r.date_start.clone()),
        datetime_str_to_i64(r.date_end.clone()),
    )
    .await;

    let insert_id = insert_market_data(
        &data.pool,
        r.exchange.to_lowercase(),
        r.symbol.to_lowercase(),
        r.market_data_type.clone(),
        datetime_str_to_i64(r.date_start.clone()),
        datetime_str_to_i64(r.date_end.clone()),
    )
    .await
    .map_err(ErrorInternalServerError)?;

    Ok(MarketDataFront {
        id: Some(insert_id),
        exchange: r.exchange.clone(),
        symbol: r.symbol.clone(),
        market_data_type: r.market_data_type.clone(),
        date_start: r.date_start.clone(),
        date_end: r.date_end.clone(),
    })
}

pub async fn market_data_dates(
    data: web::Data<AppState>,
    r: web::Query<MarketDataDatesRequest>,
) -> Result<HttpResponse, Error> {
    let dates = _market_data_dates(&data, r.into_inner()).await?;

    Ok(HttpResponse::Ok().json(dates))
}

pub async fn _market_data_dates(
    data: &web::Data<AppState>,
    r: MarketDataDatesRequest,
) -> Result<MarketDataDatesResponse, Error> {
    let dates = get_db_market_data_dates(
        &data.pool,
        &r.exchange.to_lowercase(),
        &r.symbol.to_lowercase(),
        &r.market_data_type,
    )
    .await
    .map_err(ErrorInternalServerError)?;

    let result = MarketDataDatesResponse {
        date_start: i64_to_datetime_str(dates.0),
        date_end: i64_to_datetime_str(dates.1),
    };

    Ok(result)
}
