/**
 * Example script tests gl-sdk-napi library usage for:
 * - Registration/Recovery
 * - Node creation
 * - Receiving payments
 * - Sending payments
 * 
 * This follows the same pattern as glsdk_example.py
 */

import * as fs from 'fs';
import * as crypto from 'crypto';
import {
  Scheduler,
  Signer,
  Node,
  Credentials,
  ReceiveResponse,
  SendResponse,
  OnchainReceiveResponse,
  OnchainSendResponse,
} from './index';

// Network type (matches gl-sdk Network enum)
type Network = 'bitcoin' | 'regtest';

/**
 * An application demonstrating Greenlight node operations.
 * 
 * The pattern follows the gl-sdk structure:
 * - Signer: Created from a BIP39 mnemonic phrase
 * - Scheduler: Handles node registration and recovery
 * - Credentials: Contains node authentication information
 * - Node: Main interface for Lightning operations
 */
class GreenlightApp {
  private phrase: string;
  private network: Network;
  private credentials: Credentials | null = null;
  private scheduler: Scheduler;
  private node: Node | null = null;
  private signer: Signer;

  /**
   * Initialize the Greenlight application.
   * 
   * @param phrase - BIP39 mnemonic phrase for node identity
   * @param network - Network ('bitcoin' or 'regtest')
   */
  constructor(phrase: string, network: Network) {
    this.phrase = phrase;
    this.network = network;
    this.scheduler = new Scheduler(network);
    
    // Create signer from mnemonic phrase
    this.signer = new Signer(phrase);
    const nodeId = this.signer.nodeId();
    
    console.log(`✓ Signer created for network: ${network}`);
    console.log(`✓ Node ID: ${nodeId.toString('hex')}`);
  }

  /**
   * Registers or recovers a node on Greenlight.
   * 
   * This method will:
   * 1. Try to register a new node
   * 2. If registration fails (node exists), recover instead
   * 3. Store the returned credentials for future operations
   * 
   * The credentials contain the node_id and mTLS client certificate
   * for authenticating against the node.
   */
  registerOrRecover(): void {
    try {
      console.log('Attempting to register node...');
      this.credentials = this.scheduler.register(this.signer, '');
      console.log('✓ Node registered successfully');
    } catch (e) {
      console.log(`Registration failed (node may already exist): ${e}`);
      console.log('Attempting to recover node...');
      try {
        this.credentials = this.scheduler.recover(this.signer);
        console.log('✓ Node recovered successfully');
      } catch (recoverError) {
        console.log(`✗ Recovery also failed: ${recoverError}`);
        throw recoverError;
      }
    }
  }

  /**
   * Create a Node instance using the credentials.
   * 
   * The Node is the main entrypoint to interact with the Lightning node.
   * 
   * @returns Node instance for making Lightning operations
   */
  createNode(): Node {
    if (this.credentials === null) {
      throw new Error('Must register/recover before creating node');
    }
    
    console.log('Creating node instance...');
    this.node = new Node(this.credentials);
    console.log('✓ Node created successfully');
    return this.node;
  }

  /**
   * Create a Lightning invoice to receive a payment.
   * 
   * This method generates a BOLT11 invoice that includes negotiation
   * of an LSPS2 / JIT channel, meaning that if there is no channel
   * sufficient to receive the requested funds, the node will negotiate
   * an opening.
   * 
   * @param label - Unique label for the invoice
   * @param description - Invoice description
   * @param amountMsat - Optional amount in millisatoshis (null for any amount)
   * @returns ReceiveResponse containing the BOLT11 invoice string
   */
  receive(
    label: string,
    description: string,
    amountMsat: number | null = null
  ): ReceiveResponse {
    if (this.node === null) {
      this.createNode();
    }
    
    console.log(`Creating invoice: ${amountMsat ?? 'any'} msat - '${description}'`);
    
    const invoice = this.node!.receive(label, description, amountMsat);
    console.log('✓ Invoice created successfully');
    return invoice;
  }

  /**
   * Pay a Lightning invoice.
   * 
   * @param invoice - BOLT11 invoice string to pay
   * @param amountMsat - Optional amount in millisatoshis (for zero-amount invoices)
   * @returns SendResponse containing payment details
   */
  send(invoice: string, amountMsat: number | null = null): SendResponse {
    if (this.node === null) {
      this.createNode();
    }
    
    console.log(`Paying invoice: ${invoice.substring(0, 50)}...`);
    
    const payment = this.node!.send(invoice, amountMsat);
    console.log('✓ Payment sent successfully');
    return payment;
  }

  /**
   * Get an on-chain address to receive Bitcoin.
   * 
   * @returns OnchainReceiveResponse containing the Bitcoin address
   */
  onchainReceive(): OnchainReceiveResponse {
    if (this.node === null) {
      this.createNode();
    }
    
    console.log('Generating on-chain receive address...');
    
    const response = this.node!.onchainReceive();
    console.log('✓ On-chain address generated');
    return response;
  }

  /**
   * Send Bitcoin on-chain.
   * 
   * @param destination - Bitcoin address to send to
   * @param amountOrAll - Amount in satoshis (e.g., '10000sat') or 'all' to send all funds
   * @returns OnchainSendResponse containing transaction details
   */
  onchainSend(destination: string, amountOrAll: string): OnchainSendResponse {
    if (this.node === null) {
      this.createNode();
    }
    
    console.log(`Sending on-chain: ${amountOrAll} to ${destination}`);
    
    const response = this.node!.onchainSend(destination, amountOrAll);
    console.log('✓ On-chain transaction sent');
    return response;
  }

  /**
   * Stop the node if it is currently running.
   */
  stopNode(): void {
    if (this.node !== null) {
      console.log('Stopping node...');
      this.node.stop();
      console.log('✓ Node stopped');
    }
  }

  /**
   * Save credentials to a file.
   * 
   * @param filepath - Path to save credentials
   */
  saveCredentials(filepath: string): void {
    if (this.credentials === null) {
      throw new Error('No credentials to save');
    }
    
    try {
      const credsBytes = this.credentials.save();
      fs.writeFileSync(filepath, credsBytes);
      console.log(`✓ Credentials saved to ${filepath}`);
    } catch (e) {
      console.log(`✗ Failed to save credentials: ${e}`);
      throw e;
    }
  }

  /**
   * Load credentials from a file.
   * 
   * @param filepath - Path to load credentials from
   * @returns Loaded credentials
   */
  loadCredentials(filepath: string): Credentials {
    try {
      const credsBytes = fs.readFileSync(filepath);
      this.credentials = Credentials.load(credsBytes);
      console.log(`✓ Credentials loaded from ${filepath}`);
      return this.credentials;
    } catch (e) {
      console.log(`✗ Failed to load credentials: ${e}`);
      throw e;
    }
  }
}

/**
 * Main demonstration function.
 */
function main(): void {
  console.log('='.repeat(70));
  console.log('GL-SDK-NAPI Example: Register, Create Node, and Create Invoice');
  console.log('Inspired by glsdk_example.py pattern');
  console.log('='.repeat(70));
  console.log();
  
  // Configuration
  // NOTE: These should be persisted and loaded from disk in production
  // Default test mnemonic (DO NOT USE IN PRODUCTION)
  const phrase = process.env.MNEMONIC || 
    'abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about';
  const network: Network = 'regtest'; // Options: 'bitcoin', 'regtest'
  
  // Step 1: Initialize application
  console.log('Step 1: Initializing Application');
  console.log('-'.repeat(70));
  const app = new GreenlightApp(phrase, network);
  console.log();
  
  // Step 2: Register or recover node
  console.log('Step 2: Register or Recover Node');
  console.log('-'.repeat(70));
  try {
    app.registerOrRecover();
  } catch (e) {
    console.log(`✗ Failed to register/recover: ${e}`);
    console.log('Note: This may fail without a proper Greenlight environment');
    console.error(e);
    return;
  }
  console.log();
  
  // Step 3: Create the node
  console.log('Step 3: Creating Node');
  console.log('-'.repeat(70));
  try {
    app.createNode();
  } catch (e) {
    console.log(`✗ Failed to create node: ${e}`);
    console.error(e);
    return;
  }
  console.log();
  
  // Step 4: Create an invoice (receive payment)
  console.log('Step 4: Creating Invoice (Receive)');
  console.log('-'.repeat(70));
  try {
    const randomLabel = `test-invoice-${crypto.randomBytes(4).toString('hex')}`;
    const invoiceResponse = app.receive(
      randomLabel,
      'Test payment for GL-SDK-NAPI demo',
      100000
    );
    console.log(`Invoice response: ${JSON.stringify(invoiceResponse)}`);
  } catch (e) {
    console.log(`✗ Failed to create invoice: ${e}`);
    console.error(e);
  }
  console.log();
  
  // Step 5: Generate on-chain receive address
  console.log('Step 5: On-chain Receive Address');
  console.log('-'.repeat(70));
  try {
    const onchainResponse = app.onchainReceive();
    console.log(`Onchain receive Bech32: ${onchainResponse.bech32}, P2TR: ${onchainResponse.p2Tr}`);
  } catch (e) {
    console.log(`✗ Failed to get on-chain address: ${e}`);
    console.error(e);
  }
  console.log();
  
  // Step 6: Save credentials (optional)
  console.log('Step 6: Saving Credentials');
  console.log('-'.repeat(70));
  try {
    app.saveCredentials('/tmp/glsdk_credentials.bin');
  } catch (e) {
    console.log(`Note: Credential saving: ${e}`);
  }
  console.log();
  
  // Step 7: Stop the node
  console.log('Step 7: Stopping Node');
  console.log('-'.repeat(70));
  try {
    app.stopNode();
  } catch (e) {
    console.log(`Note: Node stop: ${e}`);
  }
  console.log();
  
  console.log('='.repeat(70));
  console.log('Test Complete!');
  console.log('='.repeat(70));
}

// Run the example
try {
  main();
} catch (e) {
  console.log(`\n✗ Unexpected error: ${e}`);
  console.error(e);
}
