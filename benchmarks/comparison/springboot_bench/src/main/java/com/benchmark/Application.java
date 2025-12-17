package com.benchmark;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.Map;

@SpringBootApplication
@RestController
public class Application {

    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }

    @GetMapping("/json")
    public Map<String, String> json() {
        return Map.of("message", "Hello, World!");
    }

    @GetMapping("/plaintext")
    public String plaintext() {
        return "Hello, World!";
    }
}

