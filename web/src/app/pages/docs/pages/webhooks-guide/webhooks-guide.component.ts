import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-webhooks-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class WebhooksGuideComponent {
  page: DocPage = {
    title: 'Webhooks',
    subtitle: 'Send and receive webhooks with signature verification, retries, and delivery tracking.',
    icon: '🔔',
    badge: 'Real-Time',
    features: [
      { icon: '✅', title: 'Signature Verification', description: 'HMAC-SHA256 validation' },
      { icon: '🔄', title: 'Automatic Retries', description: 'Exponential backoff' },
      { icon: '📊', title: 'Delivery Tracking', description: 'Monitor webhook status' },
      { icon: '🎯', title: 'Event Types', description: 'Subscribe to specific events' }
    ],
    sections: [
      {
        id: 'receiving',
        title: 'Receiving Webhooks',
        content: `<p>Create an endpoint to receive webhooks from external services:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature::prelude::*;
use armature_webhook::*;

#[controller("/webhooks")]
struct WebhookController {
    webhook_service: WebhookService,
}

impl WebhookController {
    #[post("/stripe")]
    async fn stripe_webhook(
        &self,
        req: HttpRequest,
        body: Bytes,
    ) -> Result<StatusCode, Error> {
        // Verify signature
        let signature = req.header("Stripe-Signature")
            .ok_or(Error::BadRequest("Missing signature"))?;

        let secret = std::env::var("STRIPE_WEBHOOK_SECRET")?;

        self.webhook_service
            .verify_signature(&body, signature, &secret)
            .map_err(|_| Error::BadRequest("Invalid signature"))?;

        // Parse and handle event
        let event: StripeEvent = serde_json::from_slice(&body)?;

        match event.event_type.as_str() {
            "payment_intent.succeeded" => {
                handle_payment_succeeded(event.data).await?;
            }
            "customer.subscription.deleted" => {
                handle_subscription_cancelled(event.data).await?;
            }
            _ => {
                // Ignore unknown events
            }
        }

        Ok(StatusCode::OK)
    }
}`
          }
        ]
      },
      {
        id: 'sending',
        title: 'Sending Webhooks',
        content: `<p>Send webhooks to your users' endpoints:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_webhook::*;

#[injectable]
pub struct NotificationService {
    webhook_sender: WebhookSender,
}

impl NotificationService {
    pub async fn notify_order_completed(&self, order: &Order) -> Result<(), Error> {
        // Get subscriber endpoints
        let endpoints = get_webhook_endpoints(order.user_id).await?;

        for endpoint in endpoints {
            self.webhook_sender.send(WebhookPayload {
                url: endpoint.url,
                secret: endpoint.secret,
                event_type: "order.completed".into(),
                data: serde_json::to_value(order)?,
                // Retry up to 5 times with exponential backoff
                retry_config: RetryConfig::default(),
            }).await?;
        }

        Ok(())
    }
}`
          }
        ]
      },
      {
        id: 'signature',
        title: 'Signature Verification',
        content: `<p>Verify webhook signatures to ensure authenticity:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use hmac::{Hmac, Mac};
use sha2::Sha256;

pub fn verify_webhook_signature(
    payload: &[u8],
    signature: &str,
    secret: &str,
    timestamp: i64,
) -> Result<(), WebhookError> {
    // Check timestamp to prevent replay attacks
    let now = Utc::now().timestamp();
    if (now - timestamp).abs() > 300 {
        return Err(WebhookError::Expired);
    }

    // Compute expected signature
    let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())?;
    mac.update(signed_payload.as_bytes());

    // Compare signatures
    let expected = hex::encode(mac.finalize().into_bytes());
    if signature != expected {
        return Err(WebhookError::InvalidSignature);
    }

    Ok(())
}`
          }
        ]
      },
      {
        id: 'retries',
        title: 'Retry Configuration',
        content: `<p>Configure automatic retries for failed deliveries:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `let sender = WebhookSender::builder()
    .retry_config(RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_secs(60),
        max_delay: Duration::from_hours(24),
        backoff_multiplier: 2.0,
    })
    .timeout(Duration::from_secs(30))
    .build();

// Retry schedule:
// Attempt 1: Immediate
// Attempt 2: 1 minute later
// Attempt 3: 2 minutes later
// Attempt 4: 4 minutes later
// Attempt 5: 8 minutes later`
          }
        ]
      },
      {
        id: 'delivery-tracking',
        title: 'Delivery Tracking',
        content: `<p>Track webhook delivery status:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `// Record delivery attempts
#[derive(Debug)]
pub struct WebhookDelivery {
    pub id: String,
    pub endpoint_url: String,
    pub event_type: String,
    pub status: DeliveryStatus,
    pub attempts: Vec<DeliveryAttempt>,
    pub created_at: DateTime<Utc>,
}

pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed,
    Retrying,
}

// Query delivery history
let deliveries = webhook_service
    .list_deliveries(user_id)
    .filter(|d| d.status == DeliveryStatus::Failed)
    .limit(50)
    .await?;

// Manually retry a failed delivery
webhook_service.retry_delivery(delivery_id).await?;`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Always verify signatures</strong> — Never trust unsigned webhooks</li>
          <li><strong>Return 200 quickly</strong> — Process async to avoid timeouts</li>
          <li><strong>Implement idempotency</strong> — Handle duplicate deliveries</li>
          <li><strong>Log all webhook events</strong> — Debugging is critical</li>
          <li><strong>Set reasonable timeouts</strong> — Don't wait forever for endpoints</li>
          <li><strong>Provide endpoint testing</strong> — Let users verify their setup</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'queue-guide', title: 'Job Queues', description: 'Async webhook processing' },
      { id: 'websocket-sse', title: 'WebSockets', description: 'Real-time alternatives' }
    ],
    seeAlso: [
      { title: 'Security', id: 'security-guide' },
      { title: 'Rate Limiting', id: 'rate-limiting' }
    ]
  };
}

