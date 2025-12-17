package main

import (
	"os"

	"github.com/gofiber/fiber/v2"
)

type Message struct {
	Message string `json:"message"`
}

func main() {
	app := fiber.New(fiber.Config{
		DisableStartupMessage: true,
	})

	app.Get("/json", func(c *fiber.Ctx) error {
		return c.JSON(Message{Message: "Hello, World!"})
	})

	app.Get("/plaintext", func(c *fiber.Ctx) error {
		return c.SendString("Hello, World!")
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8091"
	}

	println("Fiber benchmark server starting on port", port)
	app.Listen(":" + port)
}

