use armature::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct LoginForm {
    email: String,
    password: String,
    remember_me: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ContactForm {
    name: String,
    email: String,
    subject: String,
    message: String,
}

#[injectable]
#[derive(Clone, Default)]
struct FormService;

impl FormService {
    fn new() -> Self {
        Self
    }

    fn generate_form_page(&self) -> String {
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>Form Processing Demo</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 800px; margin: 50px auto; padding: 20px; }
        form { background: #f0f0f0; padding: 20px; border-radius: 5px; margin: 20px 0; }
        input, textarea { width: 100%; padding: 10px; margin: 10px 0; box-sizing: border-box; }
        button { padding: 10px 20px; background: #007bff; color: white; border: none; cursor: pointer; }
        .result { background: #e0ffe0; padding: 15px; margin: 20px 0; border-radius: 5px; }
    </style>
</head>
<body>
    <h1>📝 Form Processing Demo</h1>

    <h2>1. Login Form (URL-encoded)</h2>
    <form action="/api/login" method="POST">
        <input type="email" name="email" placeholder="Email" required />
        <input type="password" name="password" placeholder="Password" required />
        <label>
            <input type="checkbox" name="remember_me" value="true" />
            Remember me
        </label>
        <br><br>
        <button type="submit">Login</button>
    </form>

    <h2>2. Contact Form (URL-encoded)</h2>
    <form action="/api/contact" method="POST">
        <input type="text" name="name" placeholder="Your Name" required />
        <input type="email" name="email" placeholder="Your Email" required />
        <input type="text" name="subject" placeholder="Subject" required />
        <textarea name="message" rows="5" placeholder="Message" required></textarea>
        <button type="submit">Send Message</button>
    </form>

    <h2>3. File Upload (Multipart)</h2>
    <form action="/api/upload" method="POST" enctype="multipart/form-data">
        <input type="text" name="title" placeholder="File Title" required />
        <input type="text" name="description" placeholder="Description" />
        <input type="file" name="file" required />
        <br><br>
        <button type="submit">Upload File</button>
    </form>

    <h2>4. Multiple Files Upload</h2>
    <form action="/api/upload-multiple" method="POST" enctype="multipart/form-data">
        <input type="text" name="album_name" placeholder="Album Name" required />
        <input type="file" name="files" multiple required />
        <br><br>
        <button type="submit">Upload Files</button>
    </form>
</body>
</html>
        "#.to_string()
    }
}

#[controller("/api")]
struct FormController {
    service: FormService,
}

#[module(
    providers: [FormService],
    controllers: [FormController]
)]
#[derive(Default)]
struct AppModule;

impl FormController {
    #[get("/")]
    async fn show_forms(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let html = self.service.generate_form_page();
        Ok(HttpResponse::ok().with_body(html.into_bytes()))
    }

    #[post("/login")]
    async fn login(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse URL-encoded form data into struct
        let form: LoginForm = req.form()?;
        
        println!("📧 Login attempt:");
        println!("  Email: {}", form.email);
        println!("  Remember me: {:?}", form.remember_me.as_deref().unwrap_or("false"));
        
        Ok(HttpResponse::ok()
            .with_json(&serde_json::json!({
                "status": "success",
                "message": "Login successful",
                "email": form.email,
                "remember_me": form.remember_me.is_some()
            }))?)
    }

    #[post("/contact")]
    async fn contact(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse URL-encoded form data
        let form: ContactForm = req.form()?;
        
        println!("💌 Contact form submitted:");
        println!("  Name: {}", form.name);
        println!("  Email: {}", form.email);
        println!("  Subject: {}", form.subject);
        println!("  Message: {}", form.message);
        
        Ok(HttpResponse::ok()
            .with_json(&serde_json::json!({
                "status": "success",
                "message": "Thank you for your message!",
                "data": {
                    "name": form.name,
                    "subject": form.subject
                }
            }))?)
    }

    #[post("/upload")]
    async fn upload(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse multipart form data
        let fields = req.multipart()?;
        
        let mut title = String::new();
        let mut description = String::new();
        let mut file_info = None;
        
        for field in fields {
            match field.name.as_str() {
                "title" => title = field.value.unwrap_or_default(),
                "description" => description = field.value.unwrap_or_default(),
                "file" => {
                    if let Some(file) = field.file {
                        println!("📁 File uploaded:");
                        println!("  Filename: {}", file.filename);
                        println!("  Content-Type: {}", file.content_type);
                        println!("  Size: {} bytes", file.size);
                        println!("  Extension: {:?}", file.extension());
                        println!("  Is image: {}", file.is_image());
                        
                        file_info = Some(serde_json::json!({
                            "filename": file.filename,
                            "content_type": file.content_type,
                            "size": file.size,
                            "is_image": file.is_image()
                        }));
                    }
                }
                _ => {}
            }
        }
        
        Ok(HttpResponse::ok()
            .with_json(&serde_json::json!({
                "status": "success",
                "message": "File uploaded successfully",
                "title": title,
                "description": description,
                "file": file_info
            }))?)
    }

    #[post("/upload-multiple")]
    async fn upload_multiple(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse multipart form data
        let fields = req.multipart()?;
        
        let mut album_name = String::new();
        let mut files_info = Vec::new();
        
        for field in fields {
            match field.name.as_str() {
                "album_name" => album_name = field.value.unwrap_or_default(),
                "files" => {
                    if let Some(file) = field.file {
                        println!("📁 File {}: {} ({} bytes)", files_info.len() + 1, file.filename, file.size);
                        
                        files_info.push(serde_json::json!({
                            "filename": file.filename,
                            "content_type": file.content_type,
                            "size": file.size
                        }));
                    }
                }
                _ => {}
            }
        }
        
        println!("📦 Album '{}' uploaded with {} files", album_name, files_info.len());
        
        Ok(HttpResponse::ok()
            .with_json(&serde_json::json!({
                "status": "success",
                "message": format!("Uploaded {} files to album '{}'", files_info.len(), album_name),
                "album_name": album_name,
                "files_count": files_info.len(),
                "files": files_info
            }))?)
    }

    #[post("/form-map")]
    async fn form_map_example(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse form data into HashMap (useful for dynamic forms)
        let form_data = req.form_map()?;
        
        println!("📋 Form data (HashMap):");
        for (key, value) in &form_data {
            println!("  {}: {}", key, value);
        }
        
        Ok(HttpResponse::ok()
            .with_json(&serde_json::json!({
                "status": "success",
                "fields": form_data
            }))?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Armature Form Processing Example");
    println!("====================================\n");

    println!("Form Processing Features:");
    println!("  ✓ URL-encoded form parsing (.form<T>())");
    println!("  ✓ HashMap form parsing (.form_map())");
    println!("  ✓ Multipart form data (.multipart())");
    println!("  ✓ File upload support");
    println!("  ✓ Multiple file uploads");
    println!("  ✓ Type-safe form deserialization");
    println!("\nStarting server on http://localhost:3000\n");

    println!("Try it out:");
    println!("  1. Visit http://localhost:3000/api/");
    println!("     → Interactive form demo page");
    println!("\n  2. Submit login form:");
    println!("     → Parses URL-encoded data into LoginForm struct");
    println!("\n  3. Upload files:");
    println!("     → Parses multipart/form-data");
    println!("     → Extracts file metadata and content\n");

    let app = Application::create::<AppModule>();
    app.listen(3000).await?;

    Ok(())
}

