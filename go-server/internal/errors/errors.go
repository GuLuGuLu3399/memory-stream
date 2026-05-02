// Package errors provides error types, response formatting, and recovery middleware for the API.
package errors

// AppError represents a structured API error with HTTP and business codes.
type AppError struct {
	HTTPCode   int    `json:"-"`
	BizCode    int    `json:"code"`
	Message    string `json:"message"`
	LogDetails string `json:"-"`
}

// Error returns the log details if set, otherwise the user-facing message.
func (e *AppError) Error() string {
	if e.LogDetails != "" {
		return e.LogDetails
	}
	return e.Message
}

// NewBadRequest returns a 400 AppError.
func NewBadRequest(msg string) *AppError {
	return &AppError{HTTPCode: 400, BizCode: 40001, Message: msg}
}

// NewUnauthorized returns a 401 AppError.
func NewUnauthorized(msg string) *AppError {
	return &AppError{HTTPCode: 401, BizCode: 40101, Message: msg}
}

// NewForbidden returns a 403 AppError.
func NewForbidden(msg string) *AppError {
	return &AppError{HTTPCode: 403, BizCode: 40301, Message: msg}
}

// NewNotFound returns a 404 AppError.
func NewNotFound(msg string) *AppError {
	return &AppError{HTTPCode: 404, BizCode: 40401, Message: msg}
}

// NewInternal returns a 500 AppError wrapping the original error for logging.
func NewInternal(err error) *AppError {
	return &AppError{HTTPCode: 500, BizCode: 50001, Message: "服务器开小差了", LogDetails: err.Error()}
}
