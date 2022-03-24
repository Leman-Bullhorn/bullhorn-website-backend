use rocket::response::Responder;
use rocket::serde::json::Json;
use rocket::Response;
use serde::Serialize;

#[derive(Serialize)]
struct PageInfo {
    page: i64,
    limit: i64,
}

#[derive(Serialize)]
struct PaginatedContent<T: Serialize> {
    next: Option<PageInfo>,
    previous: Option<PageInfo>,
    content: T,
}

pub struct Paginated<T> {
    content: T,
    limit: i64,
    page: i64,
    items: i64,
}

impl<T> Paginated<T> {
    pub fn new(content: T, limit: i64, page: i64, items: i64) -> Paginated<T> {
        Paginated {
            content,
            limit,
            page,
            items,
        }
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for Paginated<T> {
    fn respond_to(self, request: &'r rocket::Request<'_>) -> rocket::response::Result<'static> {
        let first_page = 1;
        let last_page = div_ceil(self.items, self.limit);

        let has_previous = self.page > first_page;
        let has_next = self.page < last_page;

        let previous = if has_previous {
            if self.page > last_page {
                Some(PageInfo {
                    page: last_page,
                    limit: self.limit,
                })
            } else {
                Some(PageInfo {
                    page: self.page - 1,
                    limit: self.limit,
                })
            }
        } else {
            None
        };

        let next = if has_next {
            Some(PageInfo {
                page: self.page + 1,
                limit: self.limit,
            })
        } else {
            None
        };

        let res = PaginatedContent {
            content: self.content,
            previous,
            next,
        };

        Response::build_from(Json(res).respond_to(request)?).ok()
    }
}

#[inline]
/// Just steal this from the std lib because its currently unstable
///
/// <https://github.com/rust-lang/rust/issues/88581>
const fn div_ceil(lhs: i64, rhs: i64) -> i64 {
    let d = lhs / rhs;
    let r = lhs % rhs;
    if (r > 0 && rhs > 0) || (r < 0 && rhs < 0) {
        d + 1
    } else {
        d
    }
}
