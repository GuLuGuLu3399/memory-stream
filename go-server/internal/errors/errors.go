package errors

type AppError struct {
	HTTPCode   int    `json:"-"`
	BizCode    int    `json:"code"`
	Message    string `json:"message"`
	LogDetails string `json:"-"`
}

func (e *AppError) Error() string {
	if e.LogDetails != "" {
		return e.LogDetails
	}
	return e.Message
}

func NewBadRequest(msg string) *AppError {
	return &AppError{HTTPCode: 400, BizCode: 40001, Message: msg}
}

func NewBadRequestWithLog(msg string, logDetail string) *AppError {
	return &AppError{HTTPCode: 400, BizCode: 40001, Message: msg, LogDetails: logDetail}
}

func NewUnauthorized(msg string) *AppError {
	return &AppError{HTTPCode: 401, BizCode: 40101, Message: msg}
}

func NewForbidden(msg string) *AppError {
	return &AppError{HTTPCode: 403, BizCode: 40301, Message: msg}
}

func NewNotFound(msg string) *AppError {
	return &AppError{HTTPCode: 404, BizCode: 40401, Message: msg}
}

func NewInternal(err error) *AppError {
	return &AppError{HTTPCode: 500, BizCode: 50001, Message: "服务器开小差了", LogDetails: err.Error()}
}

func Wrap(err error, httpCode, bizCode int, msg string) *AppError {
	return &AppError{HTTPCode: httpCode, BizCode: bizCode, Message: msg, LogDetails: err.Error()}
}
