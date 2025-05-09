pub mod ad_service;
pub mod ad_service_impl;
pub mod ad_service_factory;

pub use ad_service::AdvertisementService;
pub use ad_service_impl::AdvertisementServiceImpl;
pub use ad_service_factory::new_advertisement_service;