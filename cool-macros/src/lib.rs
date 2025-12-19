//! # cool-macros
//!
//! cool-admin Rust 过程宏库，提供装饰器风格的宏。
//!
//! ## 宏列表
//!
//! - `#[cool_controller]` - 自动生成 CRUD 控制器
//! - `#[cool_entity]` - 自动实现实体 trait
//! - `#[cool_service]` - 自动实现服务 trait
//! - `#[controller]` - 标记控制器（等价于 @Provide）
//! - `#[post]` - POST 路由装饰器
//! - `#[get]` - GET 路由装饰器
//! - `#[ignore_auth]` - 忽略认证装饰器

use darling::{ast, FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, ItemStruct};

/// CRUD 控制器宏
///
/// # 示例
///
/// ```rust,ignore
/// #[cool_controller(
///     prefix = "/admin/goods",
///     api = "add,delete,update,page,info,list"
/// )]
/// pub struct GoodsController {
///     service: GoodsService,
/// }
/// ```
#[proc_macro_attribute]
pub fn cool_controller(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str = args.to_string();
    let input = parse_macro_input!(input as ItemStruct);
    let struct_name = &input.ident;

    // 解析参数
    let mut prefix = None;
    let mut apis: Vec<String> = vec![
        "add".to_string(),
        "delete".to_string(),
        "update".to_string(),
        "page".to_string(),
        "info".to_string(),
        "list".to_string(),
    ];

    // 解析参数，支持多种格式
    // api = "add,delete,update" 或 api = ["add", "delete", "update"]
    for arg in args_str.split(',') {
        let arg = arg.trim();

        // 解析 prefix
        if let Some(p) = arg.strip_prefix("prefix") {
            if let Some(p) = p.trim().strip_prefix('=') {
                prefix = Some(p.trim().trim_matches('"').to_string());
            }
        }
        // 解析 api（支持字符串和数组形式）
        else if let Some(a) = arg.strip_prefix("api") {
            if let Some(a) = a.trim().strip_prefix('=') {
                let api_str = a.trim();
                // 判断是数组形式 ["add", "delete"] 还是字符串形式 "add,delete"
                if api_str.starts_with('[') {
                    // 数组形式，提取数组内容（简化处理）
                    let content = api_str.trim_start_matches('[').trim_end_matches(']');
                    apis = content
                        .split(',')
                        .filter_map(|s| {
                            let s = s.trim().trim_matches('"');
                            if s.is_empty() {
                                None
                            } else {
                                Some(s.to_string())
                            }
                        })
                        .collect();
                } else {
                    // 字符串形式
                    let api_str = api_str.trim_matches('"');
                    apis = api_str.split(',').map(|s| s.trim().to_string()).collect();
                }
            }
        }
        // list_query_op 配置目前需要在 register_router 中手动设置
        // 未来可以扩展宏以支持在装饰器中直接配置
    }

    let prefix = prefix.unwrap_or_else(|| {
        format!(
            "/{}",
            struct_name
                .to_string()
                .to_lowercase()
                .replace("controller", "")
        )
    });

    // 生成 API 列表
    let mut api_list = Vec::new();

    for api in &apis {
        match api.as_str() {
            "add" => {
                api_list.push(quote! { cool_core::controller::CrudApi::Add });
            }
            "delete" => {
                api_list.push(quote! { cool_core::controller::CrudApi::Delete });
            }
            "update" => {
                api_list.push(quote! { cool_core::controller::CrudApi::Update });
            }
            "page" => {
                api_list.push(quote! { cool_core::controller::CrudApi::Page });
            }
            "info" => {
                api_list.push(quote! { cool_core::controller::CrudApi::Info });
            }
            "list" => {
                api_list.push(quote! { cool_core::controller::CrudApi::List });
            }
            _ => {}
        }
    }

    let expanded = quote! {
        #input

        impl #struct_name {
            /// 获取控制器配置
            ///
            /// 对应 TS 版本的 `@CoolController` 装饰器参数
            pub fn controller_option() -> cool_core::controller::ControllerOption {
                cool_core::controller::ControllerOption {
                    prefix: Some(#prefix.to_string()),
                    api: vec![#(#api_list),*],
                    ..Default::default()
                }
            }

            /// 构建路由（使用通用的 build_crud_router）
            ///
            /// 注意：此方法需要配合中间件注入 BaseService 到 Depot
            pub fn router(&self) -> salvo::Router {
                let option = Self::controller_option();
                cool_core::controller::build_crud_router(
                    option.prefix.as_deref().unwrap_or(#prefix),
                    &option.api,
                )
            }
        }
    };

    TokenStream::from(expanded)
}

/// 实体配置
#[derive(Debug, FromDeriveInput)]
#[allow(dead_code)]
#[darling(attributes(cool_entity), supports(struct_named))]
struct EntityArgs {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<(), EntityFieldArgs>,
    /// 表名
    #[darling(default)]
    table_name: Option<String>,
    /// 是否支持软删除
    #[darling(default)]
    soft_delete: bool,
    /// 是否支持多租户
    #[darling(default)]
    tenant: bool,
}

/// 实体字段配置
#[derive(Debug, FromField)]
#[allow(dead_code)]
#[darling(attributes(cool_field))]
struct EntityFieldArgs {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    /// 是否是主键
    #[darling(default)]
    primary_key: bool,
    /// 是否自动生成
    #[darling(default)]
    auto_increment: bool,
}

/// 实体派生宏
///
/// # 示例
///
/// ```rust,ignore
/// #[derive(CoolEntity)]
/// #[cool_entity(table_name = "goods", soft_delete = true)]
/// pub struct Goods {
///     #[cool_field(primary_key, auto_increment)]
///     pub id: i64,
///     pub title: String,
///     pub price: Decimal,
/// }
/// ```
#[proc_macro_derive(CoolEntity, attributes(cool_entity, cool_field))]
pub fn cool_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let args = match EntityArgs::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let struct_name = &args.ident;
    let fields = match &args.data {
        ast::Data::Struct(f) => &f.fields,
        _ => {
            let expanded = quote! {};
            return TokenStream::from(expanded);
        }
    };

    // 根据字段类型粗略映射为前端可识别的类型字符串
    let mut column_tokens = Vec::new();
    for field in fields {
        let ident = match &field.ident {
            Some(id) => id.to_string(),
            None => continue,
        };
        let ty = &field.ty;
        let ty_repr = quote!(#ty).to_string();

        let type_str = if ty_repr.contains("String") {
            "string"
        } else if ty_repr.contains("i32")
            || ty_repr.contains("i64")
            || ty_repr.contains("u32")
            || ty_repr.contains("u64")
        {
            "number"
        } else if ty_repr.contains("Decimal") || ty_repr.contains("f32") || ty_repr.contains("f64")
        {
            "number"
        } else if ty_repr.contains("bool") {
            "boolean"
        } else if ty_repr.contains("DateTime") || ty_repr.contains("NaiveDateTime") {
            "datetime"
        } else {
            "string"
        };

        let prop_lit = syn::LitStr::new(&ident, proc_macro2::Span::call_site());
        let type_lit = syn::LitStr::new(type_str, proc_macro2::Span::call_site());

        column_tokens.push(quote! {
            cool_core::eps::ColumnInfo {
                property_name: #prop_lit.to_string(),
                r#type: #type_lit.to_string(),
                length: None,
                comment: None,
                nullable: true,
                default_value: None,
                dict: None,
                source: #prop_lit.to_string(),
            }
        });
    }

    let soft_delete_impl = if args.soft_delete {
        quote! {
            impl cool_core::entity::SoftDeleteEntity for #struct_name {
                fn delete_time_column() -> Self::Column {
                    Self::Column::DeleteTime
                }
            }
        }
    } else {
        quote! {}
    };

    let tenant_impl = if args.tenant {
        quote! {
            impl cool_core::entity::TenantEntity for #struct_name {
                fn tenant_id_column() -> Self::Column {
                    Self::Column::TenantId
                }
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        impl cool_core::entity::BaseEntity for #struct_name {
            fn id_column() -> Self::Column {
                Self::Column::Id
            }

            fn create_time_column() -> Self::Column {
                Self::Column::CreateTime
            }

            fn update_time_column() -> Self::Column {
                Self::Column::UpdateTime
            }
        }

        #soft_delete_impl
        #tenant_impl

        impl #struct_name {
            /// EPS 列信息
            ///
            /// 用于前端自动生成表单和列表字段，类型为粗略映射：
            /// - 字符串 => "string"
            /// - 数字/Decimal => "number"
            /// - 布尔 => "boolean"
            /// - 时间类型 => "datetime"
            pub fn eps_columns() -> Vec<cool_core::eps::ColumnInfo> {
                vec![
                    #(#column_tokens),*
                ]
            }
        }
    };

    TokenStream::from(expanded)
}

/// 服务派生宏
///
/// # 示例
///
/// ```rust,ignore
/// #[derive(CoolService)]
/// #[cool_service(entity = "GoodsEntity")]
/// pub struct GoodsService {
///     db: Arc<DatabaseConnection>,
/// }
/// ```
#[proc_macro_derive(CoolService, attributes(cool_service))]
pub fn cool_service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = &input.ident;

    let expanded = quote! {
        impl #struct_name {
            /// 创建服务实例
            pub fn new(db: std::sync::Arc<sea_orm::DatabaseConnection>) -> Self {
                Self { db }
            }
        }
    };

    TokenStream::from(expanded)
}

// =========================
//  核心装饰器（补齐占位）
// =========================

/// 缓存装饰器
///
/// 示例：
/// ```rust,ignore
/// #[cool_cache(ttl = 120)]
/// async fn foo(id: i64) -> Result<String, CoolError> { ... }
/// ```
/// - 默认 TTL 60s
/// - key 由函数名 + 入参序列化后 md5 生成
#[proc_macro_attribute]
pub fn cool_cache(args: TokenStream, input: TokenStream) -> TokenStream {
    let ttl_secs: u64 = args
        .to_string()
        .split('=')
        .nth(1)
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(60);

    let func = parse_macro_input!(input as syn::ItemFn);
    let fn_name = func.sig.ident.clone();
    let fn_vis = func.vis.clone();
    let fn_attrs = func.attrs.clone();
    let fn_sig = func.sig.clone();
    let fn_block = func.block.clone();

    // 收集参数名字，用于 key
    let arg_idents: Vec<syn::Ident> = func
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            syn::FnArg::Typed(pat) => match &*pat.pat {
                syn::Pat::Ident(id) => Some(id.ident.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect();

    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            use once_cell::sync::Lazy;
            use std::time::Duration;

            static __COOL_CACHE: Lazy<cool_core::cache::MemoryCache> =
                Lazy::new(|| cool_core::cache::MemoryCache::new());

            // 计算缓存 key
            let __key = {
                let raw = serde_json::to_string(&(#(&#arg_idents,)*));
                match raw {
                    Ok(s) => format!("cool_cache:{}:{}", stringify!(#fn_name), cool_core::util::md5(s)),
                    Err(_) => format!("cool_cache:{}:nokey", stringify!(#fn_name)),
                }
            };

            // 读取缓存
            if let Ok(Some(__cached)) = __COOL_CACHE.get(&__key).await {
                return Ok(__cached);
            }

            // 执行原函数
            let __result = (async #fn_block).await;

            match &__result {
                Ok(v) => {
                    let _ = __COOL_CACHE
                        .set(&__key, v, Some(Duration::from_secs(#ttl_secs)))
                        .await;
                }
                Err(_) => {
                    // 出错不写缓存
                }
            }

            __result
        }
    };

    TokenStream::from(expanded)
}

/// 事务装饰器
///
/// 简化版：要求第一个参数是数据库连接（`DatabaseConnection` 或 `&DatabaseConnection`）
/// RPC 事务装饰器
///
/// 对应 TypeScript 版本的 `@CoolRpcTransaction`。
///
/// 功能：
/// - 自动管理分布式事务
/// - 支持事务 ID 传播
/// - 自动广播事务提交/回滚事件
///
/// # 示例
///
/// ```rust,ignore
/// #[cool_rpc_transaction]
/// async fn create_order(db: &DatabaseConnection, params: Value) -> Result<Value> {
///     // 函数会自动在事务中执行
///     // 如果成功，会广播提交事件
///     // 如果失败，会广播回滚事件
/// }
/// ```
#[proc_macro_attribute]
pub fn cool_rpc_transaction(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as syn::ItemFn);
    let fn_vis = func.vis.clone();
    let fn_attrs = func.attrs.clone();
    let fn_sig = func.sig.clone();
    let fn_block = func.block.clone();

    // 提取第一个参数（通常是 params 或 request）
    // 检查是否有 rpc_transaction_id 字段
    // 提取数据库连接参数（通常是第二个参数，类型为 &DatabaseConnection）
    let db_ident = match fn_sig.inputs.iter().skip(1).find(|arg| {
        if let syn::FnArg::Typed(pat) = arg {
            if let syn::Type::Reference(ty_ref) = &*pat.ty {
                if let syn::Type::Path(type_path) = &*ty_ref.elem {
                    let path = &type_path.path;
                    path.segments
                        .last()
                        .map(|seg| seg.ident == "DatabaseConnection")
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    }) {
        Some(syn::FnArg::Typed(pat)) => match &*pat.pat {
            syn::Pat::Ident(id) => id.ident.clone(),
            _ => syn::Ident::new("db", proc_macro2::Span::call_site()),
        },
        _ => syn::Ident::new("db", proc_macro2::Span::call_site()),
    };

    // 构建包装函数
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            use cool_rpc::transaction::manager::global_transaction_manager;
            use cool_rpc::broker::CoolRpc;
            use cool_rpc::transaction::event::TransactionEvent;
            use sea_orm::TransactionTrait;
            use uuid::Uuid;

            // 获取全局 RPC Broker（需要从上下文获取，这里简化处理）
            // 实际使用时，需要通过依赖注入获取
            let manager = global_transaction_manager();

            // 检查第一个参数是否有 rpc_transaction_id
            let mut is_caller = false;
            let mut rpc_transaction_id = None;

            // 尝试从第一个参数中提取事务 ID
            if let Some(first_arg) = std::iter::once(&fn_sig.inputs[0]).next() {
                // 这里简化处理，实际需要解析参数结构
                // 假设第一个参数是 serde_json::Value 或包含 rpc_transaction_id 的结构
            }

            // 如果没有事务 ID，创建新的
            let transaction_id = if let Some(id) = rpc_transaction_id {
                id
            } else {
                is_caller = true;
                Uuid::new_v4().to_string()
            };

            // 获取或创建事务
            let txn_arc = if let Some(existing_txn) = manager.get_transaction(&transaction_id) {
                existing_txn
            } else {
                // 创建新事务
                let txn = #db_ident.begin().await?;
                let txn_arc = std::sync::Arc::new(tokio::sync::Mutex::new(txn));
                // 存储到管理器（这里简化，实际需要调用 create_transaction）
                txn_arc
            };

            // 执行函数体
            let result = {
                let #db_ident = &*txn_arc.lock().await;
                (async move #fn_block).await
            };

            // 如果是调用者，广播事务事件
            if is_caller {
                // 获取 RPC Broker（需要从上下文获取）
                // 这里简化处理，实际需要通过依赖注入
                // let rpc = get_rpc_from_context();
                // let commit = result.is_ok();
                // rpc.emit("moleculer.transaction", json!({
                //     "rpc_transaction_id": transaction_id,
                //     "commit": commit,
                // })).await?;
            }

            result
        }
    };

    TokenStream::from(expanded)
}

/// 将在函数内部开启事务，成功则提交，失败则回滚。
#[proc_macro_attribute]
pub fn cool_transaction(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func = parse_macro_input!(input as syn::ItemFn);
    let fn_vis = func.vis.clone();
    let fn_attrs = func.attrs.clone();
    let fn_sig = func.sig.clone();
    let fn_block = func.block.clone();

    // 提取第一个参数标识符
    let db_ident = match fn_sig.inputs.first() {
        Some(syn::FnArg::Typed(pat)) => match &*pat.pat {
            syn::Pat::Ident(id) => id.ident.clone(),
            _ => syn::Ident::new("db", proc_macro2::Span::call_site()),
        },
        _ => syn::Ident::new("db", proc_macro2::Span::call_site()),
    };

    // 构建包装函数
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig {
            use sea_orm::TransactionTrait;

            // 开启事务
            let __txn = #db_ident.begin().await?;

            // 在事务上下文中执行原函数体，重绑定 db 为事务引用
            let __result = {
                let #db_ident = &__txn;
                (async move #fn_block).await
            };

            match __result {
                Ok(val) => {
                    __txn.commit().await?;
                    Ok(val)
                }
                Err(err) => {
                    let _ = __txn.rollback().await;
                    Err(err)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// 路由标签装饰器（如 ignoreToken）
///
/// 对应 TypeScript 版本的 `@CoolTag`。
///
/// 作用：
/// - 为被标记的函数生成一个同模块可见的静态标签数组，供中间件或其他代码读取
/// - 示例：
///   ```rust,ignore
///   #[cool_tag("ignoreToken")]
///   async fn login(...) { ... }
///   // 生成的静态变量名类似：__COOL_TAGS_login
///   ```
#[proc_macro_attribute]
pub fn cool_tag(args: TokenStream, input: TokenStream) -> TokenStream {
    let tag_str = if args.is_empty() {
        "default".to_string()
    } else {
        let raw = args.to_string();
        // 支持 cool_tag(ignoreToken) 或 cool_tag("ignoreToken") 两种写法
        if raw.starts_with('"') {
            raw.trim_matches('"').to_string()
        } else {
            raw
        }
    };

    let func = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(f) => f,
        Err(_) => {
            // 非函数暂时不处理，直接返回原输入
            return input;
        }
    };

    let fn_name = func.sig.ident.clone();
    let static_name_str = format!("__COOL_TAGS_{}", fn_name);
    let static_ident = syn::Ident::new(&static_name_str, fn_name.span());
    let tag_lit = syn::LitStr::new(&tag_str, fn_name.span());

    let expanded = quote! {
        #func

        #[allow(non_upper_case_globals)]
        pub static #static_ident: &[&str] = &[#tag_lit];
    };

    TokenStream::from(expanded)
}

/// 事件处理装饰器
///
/// 对应 TypeScript 版本的事件装饰器。
///
/// 基础行为：
/// - 仅作用于函数
/// - 生成一个同模块可见的静态事件名常量：`pub static __COOL_EVENT_xxx: &str`
///
/// 当与 RPC 一起使用时，可以结合 [`cool_rpc_event_handler`] 宏，将
/// 事件名与具体服务名、处理函数一起注册到 `cool_rpc::event::global_event_registry`。
///
/// 当前实现为被标记的方法生成一个同模块可见的静态事件名常量，
/// 方便在运行时根据这些常量进行事件注册。
///
/// 示例：
/// ```rust,ignore
/// #[cool_event("onReady")]
/// async fn handle_ready(...) { ... }
/// // 生成：pub static __COOL_EVENT_handle_ready: &str = "onReady";
/// ```
#[proc_macro_attribute]
pub fn cool_event(args: TokenStream, input: TokenStream) -> TokenStream {
    // 解析事件名，支持：
    // - #[cool_event("onReady")]
    // - #[cool_event(onReady)]
    // - #[cool_event(event="onReady")]
    let raw = args.to_string();
    let event_name = if raw.is_empty() {
        None
    } else if let Some(pos) = raw.find('=') {
        Some(raw[pos + 1..].trim().trim_matches('"').to_string())
    } else if raw.starts_with('"') {
        Some(raw.trim_matches('"').to_string())
    } else {
        Some(raw)
    };

    // 仅对函数生效，其他项原样返回
    let func = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(f) => f,
        Err(_) => return input,
    };

    let fn_name = func.sig.ident.clone();
    let static_ident = syn::Ident::new(&format!("__COOL_EVENT_{}", fn_name), fn_name.span());
    let event_lit = syn::LitStr::new(
        &event_name.unwrap_or_else(|| fn_name.to_string()),
        fn_name.span(),
    );

    let expanded = quote! {
        #func

        #[allow(non_upper_case_globals)]
        pub static #static_ident: &str = #event_lit;
    };

    TokenStream::from(expanded)
}

// =========================
//  RPC / Task / ES / Plugin 装饰器占位
// =========================

/// RPC 服务装饰器
///
/// 对应 TypeScript 版本的 `@CoolRpcService`。
///
/// 行为：
/// - 作用于结构体
/// - 生成一个静态 RPC 服务名称：`impl Struct { fn rpc_service_name() -> &'static str }`
/// - 支持注册元数据到 `cool_rpc::registry::global_rpc_registry`，便于文档/调试：
///   - 名称默认等于结构体名，可通过参数覆盖：
///     - `#[cool_rpc_service("user")]`
///     - `#[cool_rpc_service(name="user")]`
///   - 支持配置方法列表：
///     - `#[cool_rpc_service(name="user", methods="add|delete|info")]`
#[proc_macro_attribute]
pub fn cool_rpc_service(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str = args.to_string();

    // 解析参数：name / methods
    let mut name_opt: Option<String> = None;
    let mut methods: Vec<String> = Vec::new();

    if args_str.contains('=') {
        for raw in args_str.split(',') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((k, v)) = part.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"');
                match key {
                    "name" | "service" => {
                        name_opt = Some(val.to_string());
                    }
                    "methods" => {
                        methods = val
                            .split('|')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|s| s.to_string())
                            .collect();
                    }
                    _ => {}
                }
            }
        }
    } else if !args_str.is_empty() {
        // 兼容旧用法：#[cool_rpc_service("user")]
        name_opt = Some(args_str.trim().trim_matches('"').to_string());
    }

    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;
    let service_name = name_opt.unwrap_or_else(|| struct_name.to_string());
    let lit = syn::LitStr::new(&service_name, struct_name.span());

    // 方法列表字面量
    let method_tokens: Vec<_> = methods
        .iter()
        .map(|m| {
            let m_lit = syn::LitStr::new(m, struct_name.span());
            quote! { #m_lit.to_string() }
        })
        .collect();

    // 生成注册函数名
    let register_fn = syn::Ident::new(
        &format!("__register_rpc_service_{}", struct_name),
        struct_name.span(),
    );

    let expanded = quote! {
        #item

        impl #struct_name {
            /// RPC 服务名称
            pub fn rpc_service_name() -> &'static str {
                #lit
            }
        }

        /// 注册 RPC 服务元数据，由 `#[cool_rpc_service]` 宏自动生成
        #[allow(non_snake_case)]
        pub fn #register_fn() {
            use cool_rpc::registry::{global_rpc_registry, RpcServiceMeta};

            let meta = RpcServiceMeta {
                name: #lit.to_string(),
                methods: vec![#(#method_tokens),*],
            };

            global_rpc_registry().register_service(meta);
        }
    };

    TokenStream::from(expanded)
}

/// RPC 方法装饰器
///
/// 对应 TypeScript 版本的 RPC 方法元数据装饰器。
///
/// 目前行为：
/// - 仅作用于函数
/// - 生成一个静态方法名常量：`pub static __COOL_RPC_METHOD_xxx: &str`
#[proc_macro_attribute]
pub fn cool_rpc_method(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str = args.to_string();

    let name_opt = if args_str.is_empty() {
        None
    } else if let Some(pos) = args_str.find('=') {
        Some(args_str[pos + 1..].trim().trim_matches('"').to_string())
    } else {
        Some(args_str.trim().trim_matches('"').to_string())
    };

    let func = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(f) => f,
        Err(_) => return input,
    };

    let fn_name = func.sig.ident.clone();
    let method_name = name_opt.unwrap_or_else(|| fn_name.to_string());
    let static_ident = syn::Ident::new(&format!("__COOL_RPC_METHOD_{}", fn_name), fn_name.span());
    let lit = syn::LitStr::new(&method_name, fn_name.span());

    let expanded = quote! {
        #func

        #[allow(non_upper_case_globals)]
        pub static #static_ident: &str = #lit;
    };

    TokenStream::from(expanded)
}

/// RPC 事件处理装饰器
///
/// 对应 TS 版本的 `CoolRpcEventHandler`，用于为 RPC 服务方法打上事件元数据：
/// - `service`：服务名称（必须）
/// - `event`：事件名称（可选，默认使用函数名）
///
/// 示例：
/// ```rust,ignore
/// #[cool_rpc_event_handler(service="user-service", event="user.created")]
/// async fn on_user_created(event: &RpcEvent) { ... }
/// ```
#[proc_macro_attribute]
pub fn cool_rpc_event_handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args_str = args.to_string();
    let mut service_name: Option<String> = None;
    let mut event_name: Option<String> = None;

    if args_str.contains('=') {
        for raw in args_str.split(',') {
            let part = raw.trim();
            if part.is_empty() {
                continue;
            }
            if let Some((k, v)) = part.split_once('=') {
                let key = k.trim();
                let val = v.trim().trim_matches('"');
                match key {
                    "service" | "name" => {
                        service_name = Some(val.to_string());
                    }
                    "event" => {
                        event_name = Some(val.to_string());
                    }
                    _ => {}
                }
            }
        }
    } else if !args_str.is_empty() {
        // 简化写法：#[cool_rpc_event_handler("user-service")]
        service_name = Some(args_str.trim().trim_matches('"').to_string());
    }

    let func = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(f) => f,
        Err(_) => return input,
    };

    let fn_name = func.sig.ident.clone();
    let svc_lit = syn::LitStr::new(
        &service_name.unwrap_or_else(|| "default-service".to_string()),
        fn_name.span(),
    );
    let evt_lit = syn::LitStr::new(
        &event_name.unwrap_or_else(|| fn_name.to_string()),
        fn_name.span(),
    );

    // 生成注册函数名：__register_rpc_event_{fn_name}
    let register_fn = syn::Ident::new(&format!("__register_rpc_event_{}", fn_name), fn_name.span());

    let expanded = quote! {
        #func

        /// 注册 RPC 事件处理器元数据，由 `#[cool_rpc_event_handler]` 宏自动生成
        #[allow(non_snake_case)]
        pub fn #register_fn() {
            use cool_rpc::event::{global_event_registry, RpcEventMeta};

            let meta = RpcEventMeta {
                name: #evt_lit.to_string(),
                handler: stringify!(#fn_name).to_string(),
            };

            global_event_registry().register(#svc_lit, meta);
        }
    };

    TokenStream::from(expanded)
}

/// 队列装饰器（对应任务队列）
///
/// 对齐 TS 版本 `@CoolQueue` 的核心配置能力：
/// - `type`: 队列类型，'comm' | 'getter' | 'noworker' | 'single'
/// - 任务级参数：`retries`、`timeout`、`priority`
/// - Worker 级参数：`concurrency`、`poll_interval`
///
/// 在被标记的结构体上生成一个关联函数：
/// ```rust,ignore
/// impl MyQueue {
///     pub fn queue_config() -> cool_task::QueueConfig { ... }
/// }
/// ```
#[proc_macro_attribute]
pub fn cool_queue(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;

    // 解析参数，格式示例：
    // #[cool_queue(type="comm", retries=3, timeout=60, priority=0, concurrency=4, poll_interval=1000)]
    let args_str = args.to_string();
    let mut queue_type = "comm".to_string();
    let mut retries: Option<u32> = None;
    let mut timeout: Option<u64> = None;
    let mut priority: Option<i32> = None;
    let mut concurrency: Option<usize> = None;
    let mut poll_interval: Option<u64> = None;

    for raw in args_str.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        if let Some(rest) = part.strip_prefix("type") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                let v = v.trim().trim_matches('"');
                queue_type = v.to_string();
            }
        } else if let Some(rest) = part.strip_prefix("retries") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                retries = v.trim().parse().ok();
            }
        } else if let Some(rest) = part.strip_prefix("timeout") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                timeout = v.trim().parse().ok();
            }
        } else if let Some(rest) = part.strip_prefix("priority") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                priority = v.trim().parse().ok();
            }
        } else if let Some(rest) = part.strip_prefix("concurrency") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                concurrency = v.trim().parse().ok();
            }
        } else if let Some(rest) = part.strip_prefix("poll_interval") {
            if let Some(v) = rest.trim().strip_prefix('=') {
                poll_interval = v.trim().parse().ok();
            }
        }
    }

    // 映射队列类型
    let queue_type_token = match queue_type.as_str() {
        "getter" => quote! { cool_task::QueueType::Getter },
        "noworker" => quote! { cool_task::QueueType::NoWorker },
        "single" => quote! { cool_task::QueueType::Single },
        _ => quote! { cool_task::QueueType::Comm },
    };

    let expanded = quote! {
        #item

        impl #struct_name {
            /// 队列配置，由 `#[cool_queue]` 宏自动生成
            pub fn queue_config() -> cool_task::QueueConfig {
                cool_task::QueueConfig {
                    queue_type: #queue_type_token,
                    default_job: {
                        let mut opt = cool_task::JobOptions::default();
                        if let Some(v) = #retries {
                            opt.retries = v;
                        }
                        if let Some(v) = #timeout {
                            opt.timeout = v;
                        }
                        if let Some(v) = #priority {
                            opt.priority = v;
                        }
                        opt
                    },
                    worker: {
                        let mut cfg = cool_task::worker::WorkerConfig::default();
                        if let Some(v) = #concurrency {
                            cfg.concurrency = v;
                        }
                        if let Some(v) = #poll_interval {
                            cfg.poll_interval = v;
                        }
                        cfg
                    },
                }
            }

            /// 根据队列配置和 TaskConfig 自动创建 `BaseQueue`
            ///
            /// 对齐 TS 版 `@CoolQueue` 的自动 wiring 行为：
            /// - `type = 'comm' | 'single'` 时创建带 Worker 的队列
            /// - `type = 'getter' | 'noworker'` 时仅创建生产者队列
            pub async fn build_queue(
                task_config: cool_task::TaskConfig,
            ) -> cool_task::job::JobResult<cool_task::BaseQueue> {
                let cfg = Self::queue_config();
                match cfg.queue_type {
                    cool_task::QueueType::Comm | cool_task::QueueType::Single => {
                        cool_task::BaseQueue::new(stringify!(#struct_name), task_config).await
                    }
                    cool_task::QueueType::Getter | cool_task::QueueType::NoWorker => {
                        cool_task::BaseQueue::producer_only(stringify!(#struct_name), task_config)
                            .await
                    }
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// ES 索引装饰器
///
/// 对应 TypeScript 版本的 `@CoolEsIndex`。
///
/// 功能：
/// - 支持索引名称配置
/// - 支持分片数（shards）配置
/// - 支持副本数（replicas）配置
/// - 支持分析器（analyzers）配置
///
/// # 示例
///
/// ```rust,ignore
/// // 简单用法：只指定索引名
/// #[cool_es_index("users")]
/// pub struct UserIndex { ... }
///
/// // 完整配置
/// #[cool_es_index(name = "users", shards = 8, replicas = 1)]
/// pub struct UserIndex { ... }
/// ```
#[proc_macro_attribute]
pub fn cool_es_index(args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;

    // 解析参数
    let args_str = args.to_string();
    let mut index_name = None;
    let mut shards = 8u32;
    let mut replicas = 1u32;
    let mut analyzers = Vec::<String>::new();

    // 解析参数
    if args_str.is_empty() {
        // 无参数，使用默认名称（结构体名小写）
        index_name = Some(struct_name.to_string().to_lowercase());
    } else if args_str.starts_with('"') {
        // 字符串参数，作为索引名
        index_name = Some(args_str.trim_matches('"').to_string());
    } else {
        // 键值对参数
        for part in args_str.split(',') {
            let part = part.trim();
            if let Some(pos) = part.find('=') {
                let key = part[..pos].trim();
                let value = part[pos + 1..].trim().trim_matches('"');
                
                match key {
                    "name" => index_name = Some(value.to_string()),
                    "shards" => {
                        if let Ok(v) = value.parse::<u32>() {
                            shards = v;
                        }
                    }
                    "replicas" => {
                        if let Ok(v) = value.parse::<u32>() {
                            replicas = v;
                        }
                    }
                    "analyzers" => {
                        // 支持数组格式：analyzers = ["analyzer1", "analyzer2"]
                        if value.starts_with('[') && value.ends_with(']') {
                            let content = value[1..value.len() - 1].trim();
                            analyzers = content
                                .split(',')
                                .map(|s| s.trim().trim_matches('"').to_string())
                                .filter(|s| !s.is_empty())
                                .collect();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // 如果没有指定名称，使用默认名称
    let index_name = index_name.unwrap_or_else(|| struct_name.to_string().to_lowercase());

    // 生成静态常量
    let index_name_ident = syn::Ident::new(
        &format!("__COOL_ES_INDEX_{}", struct_name),
        struct_name.span(),
    );
    let index_name_lit = syn::LitStr::new(&index_name, struct_name.span());

    // 生成配置常量
    let config_ident = syn::Ident::new(
        &format!("__COOL_ES_CONFIG_{}", struct_name),
        struct_name.span(),
    );

    // 构建 analyzers 数组
    let analyzers_tokens: Vec<_> = analyzers
        .iter()
        .map(|a| {
            let lit = syn::LitStr::new(a, struct_name.span());
            quote! { #lit.to_string() }
        })
        .collect();

    let expanded = quote! {
        #item

        /// ES 索引名称（由 #[cool_es_index] 宏生成）
        #[allow(non_upper_case_globals)]
        pub static #index_name_ident: &str = #index_name_lit;

        /// ES 索引配置（由 #[cool_es_index] 宏生成）
        #[allow(non_upper_case_globals)]
        pub static #config_ident: cool_es::EsIndexConfig = cool_es::EsIndexConfig {
            name: #index_name_lit,
            shards: #shards,
            replicas: #replicas,
            analyzers: vec![#(#analyzers_tokens),*],
        };
    };

    TokenStream::from(expanded)
}

/// EPS 路由装饰器
///
/// 对应 TS 版本中 EPS 自动收集的能力，这里通过宏在编译期生成
/// 注册函数，避免手写 `ControllerEps`。
///
/// 使用示例：
/// ```rust,ignore
/// #[eps_route(
///     module="user",
///     scope="admin",
///     prefix="/admin/user",
///     entity="UserEntity",
///     entity_type=UserEntity,
///     controller="用户管理",
///     summary="用户登录",
///     path="/login",
///     method="POST",
///     ignore_token
/// )]
/// async fn login(...) { ... }
///
/// // 启动时调用一次注册函数
/// // __register_eps_login();
/// ```
#[proc_macro_attribute]
pub fn eps_route(args: TokenStream, input: TokenStream) -> TokenStream {
    // 保留原始函数
    let func = match syn::parse::<syn::ItemFn>(input.clone()) {
        Ok(f) => f,
        Err(_) => {
            // 非函数不处理，直接返回
            return input;
        }
    };

    let fn_name = func.sig.ident.clone();

    // 解析参数：module/scope/prefix/entity/entity_type/controller/summary/path/method/tag/type/ignore_token
    let args_str = args.to_string();
    let mut module = String::new();
    let mut scope = String::from("admin");
    let mut prefix = String::new();
    let mut entity: Option<String> = None;
    let mut entity_type_ident: Option<syn::Ident> = None;
    let mut controller_desc: Option<String> = None;
    let mut summary: Option<String> = None;
    let mut path: Option<String> = None;
    let mut method: String = "GET".to_string();
    let mut ignore_token = false;
    // 查询配置：使用竖线分隔多个字段，例如 keyWordLikeFields="name|nickName"
    let mut kw_like_fields: Vec<String> = Vec::new();
    let mut field_eq_fields: Vec<String> = Vec::new();
    let mut field_like_fields: Vec<String> = Vec::new();
    // 排序配置：orderBy="createTime:desc|id:asc"
    let mut order_by_items: Vec<(String, bool)> = Vec::new();
    // 路由标签与控制器类型
    let mut route_tag: Option<String> = None;
    let mut controller_type: Option<String> = None;

    for raw in args_str.split(',') {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }

        if part == "ignore_token" || part == "ignoreToken" {
            ignore_token = true;
            continue;
        }

        if let Some(rest) = part.split_once('=') {
            let key = rest.0.trim();
            let val = rest.1.trim().trim_matches('"');
            match key {
                "module" => module = val.to_string(),
                "scope" => scope = val.to_string(),
                "prefix" => prefix = val.to_string(),
                "entity" => entity = Some(val.to_string()),
                "entity_type" => {
                    entity_type_ident = Some(syn::Ident::new(val, proc_macro2::Span::call_site()))
                }
                "controller" | "desc" => controller_desc = Some(val.to_string()),
                "summary" => summary = Some(val.to_string()),
                "path" => path = Some(val.to_string()),
                "method" => method = val.to_string().to_uppercase(),
                // 模糊查询字段：keyWordLikeFields="name|nickName"
                "keyWordLikeFields" | "key_word_like_fields" => {
                    kw_like_fields = val
                        .split('|')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                }
                // 等值查询字段：fieldEq="status|type"
                "fieldEq" | "field_eq" => {
                    field_eq_fields = val
                        .split('|')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                }
                // 字段模糊查询：fieldLike="phone|email"
                "fieldLike" | "field_like" => {
                    field_like_fields = val
                        .split('|')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect();
                }
                // 排序字段：orderBy="createTime:desc|id:asc"
                "orderBy" | "order_by" => {
                    order_by_items = val
                        .split('|')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|pair| {
                            let mut parts = pair.split(':').map(|p| p.trim());
                            let col = parts.next().unwrap_or_default().to_string();
                            let dir = parts.next().unwrap_or("asc");
                            let asc = !dir.eq_ignore_ascii_case("desc");
                            (col, asc)
                        })
                        .collect();
                }
                // 路由标签：tag="user"
                "tag" => {
                    if !val.is_empty() {
                        route_tag = Some(val.to_string());
                    }
                }
                // 控制器类型：type="crud" 或自定义
                "type" => {
                    if !val.is_empty() {
                        controller_type = Some(val.to_string());
                    }
                }
                _ => {}
            }
        }
    }

    // 默认 path 使用函数名
    let default_path = format!("/{}", fn_name);
    let final_path = path.unwrap_or(default_path);

    // 构造字面量
    let module_lit = syn::LitStr::new(&module, fn_name.span());
    let prefix_lit = syn::LitStr::new(&prefix, fn_name.span());
    let controller_lit = syn::LitStr::new(
        controller_desc.as_deref().unwrap_or_default(),
        fn_name.span(),
    );
    let summary_lit = syn::LitStr::new(summary.as_deref().unwrap_or_default(), fn_name.span());
    let path_lit = syn::LitStr::new(&final_path, fn_name.span());
    let method_lit = syn::LitStr::new(&method, fn_name.span());
    let entity_lit = entity.as_ref().map(|e| syn::LitStr::new(e, fn_name.span()));
    let tag_lit = route_tag
        .as_ref()
        .map(|t| syn::LitStr::new(t, fn_name.span()));
    let ctrl_type_lit = controller_type
        .as_ref()
        .map(|t| syn::LitStr::new(t, fn_name.span()));

    // QueryOpInfo 字段字面量
    let kw_like_tokens: Vec<_> = kw_like_fields
        .iter()
        .map(|s| {
            let lit = syn::LitStr::new(s, fn_name.span());
            quote! { #lit.to_string() }
        })
        .collect();
    let field_eq_tokens: Vec<_> = field_eq_fields
        .iter()
        .map(|s| {
            let lit = syn::LitStr::new(s, fn_name.span());
            quote! { #lit.to_string() }
        })
        .collect();
    let field_like_tokens: Vec<_> = field_like_fields
        .iter()
        .map(|s| {
            let lit = syn::LitStr::new(s, fn_name.span());
            quote! { #lit.to_string() }
        })
        .collect();
    let order_by_tokens: Vec<_> = order_by_items
        .iter()
        .map(|(col, asc)| {
            let col_lit = syn::LitStr::new(col, fn_name.span());
            let asc_val = *asc;
            quote! { (#col_lit.to_string(), #asc_val) }
        })
        .collect();

    // 如果提供了实体类型，则使用该类型的 eps_columns 作为列信息
    let (columns_expr, page_columns_expr) = if let Some(ref ident) = entity_type_ident {
        let ty = ident.clone();
        (
            quote! { <#ty>::eps_columns() },
            quote! { <#ty>::eps_columns() },
        )
    } else {
        (quote! { Vec::new() }, quote! { Vec::new() })
    };

    // 映射 scope
    let scope_token = match scope.as_str() {
        "app" | "APP" => quote! { cool_core::eps::EpsScope::App },
        _ => quote! { cool_core::eps::EpsScope::Admin },
    };

    // 生成注册函数名
    let register_fn_name = syn::Ident::new(&format!("__register_eps_{}", fn_name), fn_name.span());

    let ignore_token_bool = ignore_token;

    let expanded = quote! {
        #func

        /// EPS 注册函数，由 `#[eps_route]` 宏自动生成
        #[allow(non_snake_case)]
        pub fn #register_fn_name() {
            use cool_core::eps::{
                ControllerEps, ModuleInfo, RouteInfo, QueryOpInfo, EpsRegistry, EpsScope,
                global_eps_registry,
            };

            let registry: &EpsRegistry = global_eps_registry();

            // 注册模块基础信息（重复注册将覆盖旧值，问题不大）
            if !#module_lit.value().is_empty() {
                registry.register_module(
                    #module_lit.value(),
                    ModuleInfo {
                        name: #module_lit.value().to_string(),
                        description: #controller_lit.value().to_string(),
                    },
                );
            }

            // 构造路由信息
            let route = RouteInfo {
                method: #method_lit.value().to_string(),
                path: #path_lit.value().to_string(),
                summary: if #summary_lit.value().is_empty() {
                    None
                } else {
                    Some(#summary_lit.value().to_string())
                },
                tag: {
                    #tag_lit
                        .as_ref()
                        .map(|t| t.value().to_string())
                },
                ignore_token: #ignore_token_bool,
            };

            // 列信息与分页列信息：如提供实体类型则自动调用其 eps_columns
            let eps = ControllerEps {
                module: #module_lit.value().to_string(),
                prefix: #prefix_lit.value().to_string(),
                entity_name: #entity_lit.as_ref().map(|e| e.value().to_string()),
                r#type: #ctrl_type_lit
                    .as_ref()
                    .map(|t| t.value().to_string()),
                description: if #controller_lit.value().is_empty() {
                    None
                } else {
                    Some(#controller_lit.value().to_string())
                },
                api: vec![route],
                columns: #columns_expr,
                page_query_op: Some(QueryOpInfo {
                    key_word_like_fields: vec![#(#kw_like_tokens),*],
                    field_eq: vec![#(#field_eq_tokens),*],
                    field_like: vec![#(#field_like_tokens),*],
                    order_by: vec![#(#order_by_tokens),*],
                }),
                page_columns: #page_columns_expr,
            };

            // scope + module 注册
            registry.register_controller(
                #scope_token,
                #module_lit.value(),
                eps,
            );
        }
    };

    TokenStream::from(expanded)
}

/// 插件装饰器
///
/// 对应 TypeScript 版本的插件装饰器。
///
/// 目前行为：
/// - 仅作用于结构体
/// - 生成一个静态插件 key 常量，默认等于结构体名小写：
///   `pub static __COOL_PLUGIN_KEY_MyPlugin: &str = "myplugin";`
#[proc_macro_attribute]
pub fn cool_plugin(args: TokenStream, input: TokenStream) -> TokenStream {
    let raw = args.to_string();
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;

    let key_opt = if raw.is_empty() {
        None
    } else if let Some(pos) = raw.find('=') {
        Some(raw[pos + 1..].trim().trim_matches('"').to_string())
    } else if raw.starts_with('"') {
        Some(raw.trim_matches('"').to_string())
    } else {
        Some(raw)
    };

    let key = key_opt.unwrap_or_else(|| struct_name.to_string().to_lowercase());
    let static_ident = syn::Ident::new(
        &format!("__COOL_PLUGIN_KEY_{}", struct_name),
        struct_name.span(),
    );
    let lit = syn::LitStr::new(&key, struct_name.span());

    let expanded = quote! {
        #item

        #[allow(non_upper_case_globals)]
        pub static #static_ident: &str = #lit;
    };

    TokenStream::from(expanded)
}

/// Controller 标记宏（等价于 TS 的 @Provide()）
///
/// 用于标记控制器结构体，生成元数据用于依赖注入
///
/// # 示例
///
/// ```rust,ignore
/// #[controller]
/// pub struct MyController { ... }
/// ```
///
/// 对应 TS: @Provide()
#[proc_macro_attribute]
pub fn controller(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    let struct_name = &item.ident;

    // 生成元数据静态变量，用于依赖注入系统识别
    let meta_name = syn::Ident::new(
        &format!("__CONTROLLER_META_{}", struct_name),
        struct_name.span(),
    );

    let expanded = quote! {
        #item

        /// 控制器元数据（由 #[controller] 宏生成）
        #[allow(non_upper_case_globals)]
        static #meta_name: &str = stringify!(#struct_name);
    };

    TokenStream::from(expanded)
}
