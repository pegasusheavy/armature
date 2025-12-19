package main

import (
	"net/http"
	"os"

	"github.com/gin-gonic/gin"
)

type Message struct {
	Message string `json:"message"`
}

func main() {
	gin.SetMode(gin.ReleaseMode)
	r := gin.New()

	r.GET("/json", func(c *gin.Context) {
		c.JSON(http.StatusOK, Message{Message: "Hello, World!"})
	})

	r.GET("/plaintext", func(c *gin.Context) {
		c.String(http.StatusOK, "Hello, World!")
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8090"
	}

	println("Gin benchmark server starting on port", port)
	r.Run(":" + port)
}

