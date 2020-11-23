use gio::prelude::*;

pub mod row_data {
    use super::*;

    use glib::subclass;
    use glib::subclass::prelude::*;
    use glib::translate::*;

    mod imp {
        use super::*;
        use std::cell::RefCell;
        pub struct RowData {
            pub app_id: RefCell<Option<String>>,
            pub display_name: RefCell<Option<String>>,
            pub icon_name: RefCell<Option<String>>,
            pub keywords: RefCell<Option<String>>,
            pub categories: RefCell<Option<String>>,
        }

        static PROPERTIES: [subclass::Property; 5] = [
            subclass::Property("app-id", |name| {
                glib::ParamSpec::string(
                    name,
                    "AppId",
                    "AppId",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("display-name", |name| {
                glib::ParamSpec::string(
                    name,
                    "DisplayName",
                    "DisplayName",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("icon-name", |name| {
                glib::ParamSpec::string(
                    name,
                    "IconName",
                    "IconName",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("keywords", |name| {
                glib::ParamSpec::string(
                    name,
                    "Keywords",
                    "Keywords",
                    None,
                    glib::ParamFlags::READWRITE,
                )
            }),
            subclass::Property("categories", |name| {
                glib::ParamSpec::string(
                    name,
                    "Categories",
                    "Categories",
                    None,
                    glib::ParamFlags::READWRITE,
                )
            }),
        ];

        impl ObjectSubclass for RowData {
            const NAME: &'static str = "RowData";
            type ParentType = glib::Object;
            type Instance = subclass::simple::InstanceStruct<Self>;
            type Class = subclass::simple::ClassStruct<Self>;

            glib_object_subclass!();

            fn class_init(klass: &mut Self::Class) {
                klass.install_properties(&PROPERTIES);
            }

            fn new() -> Self {
                Self {
                    app_id: RefCell::new(None),
                    display_name: RefCell::new(None),
                    icon_name: RefCell::new(None),
                    keywords: RefCell::new(None),
                    categories: RefCell::new(None)
                }
            }
        }

        impl ObjectImpl for RowData {
            glib_object_impl!();

            fn set_property(&self, _obj: &glib::Object, id: usize, value: &glib::Value) {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("app-id", ..) => {
                        let app_id = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.app_id.replace(app_id);
                    }
                    subclass::Property("display-name", ..) => {
                        let display_name = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.display_name.replace(display_name);
                    }
                    subclass::Property("icon-name", ..) => {
                        let icon_name = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.icon_name.replace(icon_name);
                    }
                    subclass::Property("keywords", ..) => {
                        let keywords = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.keywords.replace(keywords);
                    }
                    subclass::Property("categories", ..) => {
                        let categories = value
                            .get()
                            .expect("type conformity checked by `Object::set_property`");
                        self.categories.replace(categories);
                    }
                    _ => unimplemented!(),
                }
            }

            fn get_property(&self, _obj: &glib::Object, id: usize) -> Result<glib::Value, ()> {
                let prop = &PROPERTIES[id];

                match *prop {
                    subclass::Property("app-id", ..) => Ok(self.display_name.borrow().to_value()),
                    subclass::Property("display-name", ..) => Ok(self.display_name.borrow().to_value()),
                    subclass::Property("icon-name", ..) => Ok(self.icon_name.borrow().to_value()),
                    subclass::Property("keywords", ..) => Ok(self.keywords.borrow().to_value()),
                    subclass::Property("categories", ..) => Ok(self.categories.borrow().to_value()),
                    _ => unimplemented!(),
                }
            }
        }
    }

    glib_wrapper! {
        pub struct RowData(Object<subclass::simple::InstanceStruct<imp::RowData>, subclass::simple::ClassStruct<imp::RowData>, RowDataClass>);

        match fn {
            get_type => || imp::RowData::get_type().to_glib(),
        }
    }

    impl RowData {
        pub fn new(app_id: &str, display_name: &str, icon_name: &str, keywords: &str, categories: &str) -> RowData {
            glib::Object::new(Self::static_type(), &[("app-id", &app_id), ("display-name", &display_name), 
                ("icon-name", &icon_name), ("keywords", &keywords), ("categories", &categories)])
                .expect("Failed to create row data")
                .downcast()
                .expect("Created row data is of wrong type")
        }
    }
    
    pub trait RowDataExt {
        fn match_any(&self, query: &str) -> bool;
        fn match_display_name(&self, query: &str) -> bool;
        fn match_keywords(&self, query: &str) -> bool;
        fn match_categories(&self, query: &str) -> bool;
        fn get_app_id(&self) -> String;
        fn get_icon_name(&self) -> String;
    }

    impl RowDataExt for RowData {

        fn match_any(&self, query: &str) -> bool {
            self.match_display_name(query) || self.match_keywords(query) || self.match_categories(query)
        }
        
        fn match_display_name(&self, query: &str) -> bool {
            let priv_ = imp::RowData::from_instance(self);
            if let Some(name) = &*priv_.display_name.borrow() {
                return name.to_lowercase().contains(&query.to_lowercase())
            }
            false
        }

        fn match_keywords(&self, query: &str) -> bool {
            let priv_ = imp::RowData::from_instance(self);
            if let Some(keywords) = &*priv_.keywords.borrow() {
                return keywords.to_lowercase().contains(&query.to_lowercase())
            }
            false
        }

        fn match_categories(&self, query: &str) -> bool {
            let priv_ = imp::RowData::from_instance(self);
            if let Some(keywords) = &*priv_.keywords.borrow() {
                return keywords.to_lowercase().contains(&query.to_lowercase()) 
            }
            false
        }
        fn get_app_id(&self) -> String {
            let priv_ = imp::RowData::from_instance(self);
            if let Some(app_id) = &*priv_.app_id.borrow() {
                return app_id.to_string();
            }
            "".to_string()

        }
        fn get_icon_name(&self) -> String {
            let priv_ = imp::RowData::from_instance(self);
            if let Some(icon_name) = &*priv_.icon_name.borrow() {
                return icon_name.to_string();
            }
            "".to_string()
        }
    }
}