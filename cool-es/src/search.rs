//! 搜索构建器

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// 搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 总数
    pub total: u64,
    /// 命中列表
    pub hits: Vec<SearchHit>,
    /// 聚合结果
    pub aggregations: Option<Value>,
}

/// 搜索命中
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    /// 索引
    pub index: String,
    /// 文档 ID
    pub id: String,
    /// 分数
    pub score: Option<f64>,
    /// 文档内容
    pub source: Value,
}

/// 搜索构建器
#[derive(Default)]
pub struct SearchBuilder {
    query: Option<Value>,
    from: Option<u64>,
    size: Option<u64>,
    sort: Vec<Value>,
    source: Option<Value>,
    aggregations: Option<Value>,
    highlight: Option<Value>,
}

impl SearchBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置查询
    pub fn query(mut self, query: Value) -> Self {
        self.query = Some(query);
        self
    }

    /// Match 查询
    pub fn match_query(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.query = Some(json!({
            "match": { field: value.into() }
        }));
        self
    }

    /// Term 查询
    pub fn term(mut self, field: &str, value: impl Into<Value>) -> Self {
        self.query = Some(json!({
            "term": { field: value.into() }
        }));
        self
    }

    /// Bool 查询
    pub fn bool_query(mut self, must: Vec<Value>, should: Vec<Value>, must_not: Vec<Value>) -> Self {
        let mut bool_query = json!({});
        if !must.is_empty() {
            bool_query["must"] = json!(must);
        }
        if !should.is_empty() {
            bool_query["should"] = json!(should);
        }
        if !must_not.is_empty() {
            bool_query["must_not"] = json!(must_not);
        }
        self.query = Some(json!({ "bool": bool_query }));
        self
    }

    /// 范围查询
    pub fn range(mut self, field: &str, gte: Option<Value>, lte: Option<Value>) -> Self {
        let mut range = json!({});
        if let Some(v) = gte {
            range["gte"] = v;
        }
        if let Some(v) = lte {
            range["lte"] = v;
        }
        self.query = Some(json!({
            "range": { field: range }
        }));
        self
    }

    /// 设置分页
    pub fn from(mut self, from: u64) -> Self {
        self.from = Some(from);
        self
    }

    /// 设置大小
    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    /// 添加排序
    pub fn sort(mut self, field: &str, order: &str) -> Self {
        self.sort.push(json!({ field: { "order": order } }));
        self
    }

    /// 设置返回字段
    pub fn source(mut self, fields: Vec<&str>) -> Self {
        self.source = Some(json!(fields));
        self
    }

    /// 设置聚合
    pub fn aggregation(mut self, name: &str, agg: Value) -> Self {
        let mut aggs = self.aggregations.unwrap_or(json!({}));
        aggs[name] = agg;
        self.aggregations = Some(aggs);
        self
    }

    /// 设置高亮
    pub fn highlight(mut self, fields: Vec<&str>) -> Self {
        let fields_obj: Value = fields
            .into_iter()
            .map(|f| (f.to_string(), json!({})))
            .collect::<serde_json::Map<String, Value>>()
            .into();

        self.highlight = Some(json!({ "fields": fields_obj }));
        self
    }

    /// 构建查询体
    pub fn build(self) -> Value {
        let mut body = json!({});

        if let Some(q) = self.query {
            body["query"] = q;
        } else {
            body["query"] = json!({ "match_all": {} });
        }

        if let Some(from) = self.from {
            body["from"] = json!(from);
        }

        if let Some(size) = self.size {
            body["size"] = json!(size);
        }

        if !self.sort.is_empty() {
            body["sort"] = json!(self.sort);
        }

        if let Some(source) = self.source {
            body["_source"] = source;
        }

        if let Some(aggs) = self.aggregations {
            body["aggs"] = aggs;
        }

        if let Some(highlight) = self.highlight {
            body["highlight"] = highlight;
        }

        body
    }
}

/// 从 ES 响应解析搜索结果
pub fn parse_search_result(response: Value) -> SearchResult {
    let default_hits = json!({});
    let hits_obj_ref = response.get("hits");
    let hits_obj = hits_obj_ref.unwrap_or(&default_hits);

    let total = hits_obj
        .get("total")
        .and_then(|t| t.get("value"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let hits: Vec<SearchHit> = hits_obj
        .get("hits")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .map(|hit| SearchHit {
                    index: hit.get("_index").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    id: hit.get("_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    score: hit.get("_score").and_then(|v| v.as_f64()),
                    source: hit.get("_source").cloned().unwrap_or(json!({})),
                })
                .collect()
        })
        .unwrap_or_default();

    let aggregations = response.get("aggregations").cloned();

    SearchResult {
        total,
        hits,
        aggregations,
    }
}

