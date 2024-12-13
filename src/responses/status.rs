pub struct Status;

#[allow(dead_code)]
impl Status {
    pub const CONTINUE: &str = "100 Continue";
    pub const SWITCHING_PROTOCOLS: &str = "101 Switching Protocols";
    pub const PROCESSING: &str = "102 Processing";

    pub const OK: &str = "200 OK";
    pub const CREATED: &str = "201 Created";
    pub const ACCEPTED: &str = "202 Accepted";
    pub const NON_AUTHORITATIVE_INFORMATION: &str = "203 Non Authoritative Information";
    pub const NO_CONTENT: &str = "204 No Content";
    pub const RESET_CONTENT: &str = "205 Reset Content";
    pub const PARTIAL_CONTENT: &str = "206 Partial Content";
    pub const MULTI_STATUS: &str = "207 Multi-Status";
    pub const ALREADY_REPORTED: &str = "208 Already Reported";

    pub const IM_USED: &str = "226 IM Used";

    pub const MULTIPLE_CHOICES: &str = "300 Multiple Choices";
    pub const MOVED_PERMANENTLY: &str = "301 Moved Permanently";
    pub const FOUND: &str = "302 Found";
    pub const SEE_OTHER: &str = "303 See Other";
    pub const NOT_MODIFIED: &str = "304 Not Modified";
    pub const USE_PROXY: &str = "305 Use Proxy";
    pub const TEMPORARY_REDIRECT: &str = "307 Temporary Redirect";
    pub const PERMANENT_REDIRECT: &str = "308 Permanent Redirect";

    pub const BAD_REQUEST: &str = "400 Bad Request";
    pub const UNAUTHORIZED: &str = "401 Unauthorized";
    pub const PAYMENT_REQUIRED: &str = "402 Payment Required";
    pub const FORBIDDEN: &str = "403 Forbidden";
    pub const NOT_FOUND: &str = "404 Not Found";
    pub const METHOD_NOT_ALLOWED: &str = "405 Method Not Allowed";
    pub const NOT_ACCEPTABLE: &str = "406 Not Acceptable";
    pub const PROXY_AUTHENTICATION_REQUIRED: &str = "407 Proxy Authentication Required";
    pub const REQUEST_TIMEOUT: &str = "408 Request Timeout";
    pub const CONFLICT: &str = "409 Conflict";
    pub const GONE: &str = "410 Gone";
    pub const LENGTH_REQUIRED: &str = "411 Length Required";
    pub const PRECONDITION_FAILED: &str = "412 Precondition Failed";
    pub const PAYLOAD_TOO_LARGE: &str = "413 Payload Too Large";
    pub const URI_TOO_LONG: &str = "414 URI Too Long";
    pub const UNSUPPORTED_MEDIA_TYPE: &str = "415 Unsupported Media Type";
    pub const RANGE_NOT_SATISFIABLE: &str = "416 Range Not Satisfiable";
    pub const EXPECTATION_FAILED: &str = "417 Expectation Failed";
    pub const IM_A_TEAPOT: &str = "418 I'm a teapot";

    pub const MISDIRECTED_REQUEST: &str = "421 Misdirected Request";
    pub const UNPROCESSABLE_ENTITY: &str = "422 Unprocessable Entity";
    pub const LOCKED: &str = "423 Locked";
    pub const FAILED_DEPENDENCY: &str = "424 Failed Dependency";

    pub const TOO_EARLY: &str = "425 Too Early";

    pub const UPGRADE_REQUIRED: &str = "426 Upgrade Required";

    pub const PRECONDITION_REQUIRED: &str = "428 Precondition Required";
    pub const TOO_MANY_REQUESTS: &str = "429 Too Many Requests";

    pub const REQUEST_HEADER_FIELDS_TOO_LARGE: &str = "431 Request Header Fields Too Large";

    pub const UNAVAILABLE_FOR_LEGAL_REASONS: &str = "451 Unavailable For Legal Reasons";

    pub const INTERNAL_SERVER_ERROR: &str = "500 Internal Server Error";
    pub const NOT_IMPLEMENTED: &str = "501 Not Implemented";
    pub const BAD_GATEWAY: &str = "502 Bad Gateway";
    pub const SERVICE_UNAVAILABLE: &str = "503 Service Unavailable";
    pub const GATEWAY_TIMEOUT: &str = "504 Gateway Timeout";
    pub const HTTP_VERSION_NOT_SUPPORTED: &str = "505 HTTP Version Not Supported";
    pub const VARIANT_ALSO_NEGOTIATES: &str = "506 Variant Also Negotiates";
    pub const INSUFFICIENT_STORAGE: &str = "507 Insufficient Storage";
    pub const LOOP_DETECTED: &str = "508 Loop Detected";

    pub const NOT_EXTENDED: &str = "510 Not Extended";
    pub const NETWORK_AUTHENTICATION_REQUIRED: &str = "511 Network Authentication Required";
}
