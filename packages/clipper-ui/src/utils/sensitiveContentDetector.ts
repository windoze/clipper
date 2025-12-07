import validator from "validator";

export interface SensitiveContentWarning {
  type: SensitiveContentType;
  details?: string;
}

export type SensitiveContentType =
  | "email"
  | "creditCard"
  | "phone"
  | "ssn"
  | "ipAddress"
  | "jwt"
  | "apiKey"
  | "privateKey"
  | "password"
  | "awsKey";

// Patterns for detecting sensitive content
const PATTERNS = {
  // Social Security Number (US)
  ssn: /\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b/g,

  // Phone numbers (various formats)
  phone: /\b(?:\+?1[-.\s]?)?\(?[2-9]\d{2}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b/g,

  // JWT tokens
  jwt: /\beyJ[A-Za-z0-9_-]*\.eyJ[A-Za-z0-9_-]*\.[A-Za-z0-9_-]+\b/g,

  // Generic API keys (long alphanumeric strings with common prefixes)
  apiKey:
    /\b(?:sk|pk|api|key|token|secret|access|auth)[-_]?[A-Za-z0-9]{20,}\b/gi,

  // AWS Access Key ID
  awsAccessKey: /\b(?:AKIA|ABIA|ACCA|ASIA)[A-Z0-9]{16}\b/g,

  // Private keys (PEM format)
  privateKey:
    /-----BEGIN (?:RSA |EC |DSA |OPENSSH |PGP )?PRIVATE KEY-----/gi,

  // GitHub tokens
  githubToken: /\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{36,}\b/g,

  // Slack tokens
  slackToken: /\bxox[baprs]-[A-Za-z0-9-]+\b/g,

  // Generic secrets in key=value format
  secretAssignment:
    /\b(?:password|passwd|pwd|secret|token|api_key|apikey|auth|credential)s?\s*[=:]\s*['"]?[^\s'"]{8,}['"]?/gi,

  // Bearer tokens
  bearerToken: /\bBearer\s+[A-Za-z0-9_-]+\.?[A-Za-z0-9_-]*\.?[A-Za-z0-9_-]*/gi,
};

/**
 * Detect sensitive content in a string
 * Returns an array of warnings about potentially sensitive content
 */
export function detectSensitiveContent(
  content: string
): SensitiveContentWarning[] {
  const warnings: SensitiveContentWarning[] = [];
  const seenTypes = new Set<SensitiveContentType>();

  // Check for email addresses
  const words = content.split(/\s+/);
  for (const word of words) {
    if (validator.isEmail(word) && !seenTypes.has("email")) {
      warnings.push({ type: "email" });
      seenTypes.add("email");
      break;
    }
  }

  // Check for credit card numbers
  const potentialCards = content.match(/\b[\d\s-]{13,19}\b/g) || [];
  for (const card of potentialCards) {
    const cleanCard = card.replace(/[\s-]/g, "");
    if (validator.isCreditCard(cleanCard) && !seenTypes.has("creditCard")) {
      warnings.push({ type: "creditCard" });
      seenTypes.add("creditCard");
      break;
    }
  }

  // Check for IP addresses
  const ipMatches = content.match(
    /\b(?:\d{1,3}\.){3}\d{1,3}\b|\b(?:[A-Fa-f0-9]{1,4}:){7}[A-Fa-f0-9]{1,4}\b/g
  );
  if (ipMatches) {
    for (const ip of ipMatches) {
      if (validator.isIP(ip) && !seenTypes.has("ipAddress")) {
        // Skip common non-sensitive IPs
        if (
          !ip.startsWith("127.") &&
          !ip.startsWith("0.") &&
          ip !== "255.255.255.255"
        ) {
          warnings.push({ type: "ipAddress" });
          seenTypes.add("ipAddress");
          break;
        }
      }
    }
  }

  // Check for SSN
  if (PATTERNS.ssn.test(content) && !seenTypes.has("ssn")) {
    warnings.push({ type: "ssn" });
    seenTypes.add("ssn");
  }

  // Check for phone numbers
  if (PATTERNS.phone.test(content) && !seenTypes.has("phone")) {
    warnings.push({ type: "phone" });
    seenTypes.add("phone");
  }

  // Check for JWT tokens
  if (PATTERNS.jwt.test(content) && !seenTypes.has("jwt")) {
    warnings.push({ type: "apiKey", details: "JWT token" });
    seenTypes.add("apiKey");
  }

  // Check for private keys
  if (PATTERNS.privateKey.test(content) && !seenTypes.has("privateKey")) {
    warnings.push({ type: "privateKey" });
    seenTypes.add("privateKey");
  }

  // Check for AWS keys
  if (PATTERNS.awsAccessKey.test(content) && !seenTypes.has("awsKey")) {
    warnings.push({ type: "awsKey", details: "AWS Access Key" });
    seenTypes.add("awsKey");
  }

  // Check for GitHub tokens
  if (PATTERNS.githubToken.test(content) && !seenTypes.has("apiKey")) {
    warnings.push({ type: "apiKey", details: "GitHub token" });
    seenTypes.add("apiKey");
  }

  // Check for Slack tokens
  if (PATTERNS.slackToken.test(content) && !seenTypes.has("apiKey")) {
    warnings.push({ type: "apiKey", details: "Slack token" });
    seenTypes.add("apiKey");
  }

  // Check for generic API keys
  if (PATTERNS.apiKey.test(content) && !seenTypes.has("apiKey")) {
    warnings.push({ type: "apiKey" });
    seenTypes.add("apiKey");
  }

  // Check for secret assignments (password=xxx, etc.)
  if (PATTERNS.secretAssignment.test(content) && !seenTypes.has("password")) {
    warnings.push({ type: "password" });
    seenTypes.add("password");
  }

  // Check for Bearer tokens
  if (PATTERNS.bearerToken.test(content) && !seenTypes.has("apiKey")) {
    warnings.push({ type: "apiKey", details: "Bearer token" });
    seenTypes.add("apiKey");
  }

  return warnings;
}

/**
 * Check if content contains any sensitive data
 */
export function hasSensitiveContent(content: string): boolean {
  return detectSensitiveContent(content).length > 0;
}
