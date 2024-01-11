# Axon Deployment Repository

## Introduction

This repository serves as the storage directory for Axon deployment files. It encompasses Axon Kubernetes deployment, node deployment, and private key generation methods. The following sections provide detailed instructions for various deployment methods. It is strongly recommended to generate your own key regardless of the deployment method chosen.

## Deployment Methods

### 1. Private Key Generation

- Steps to generate a private key for securing your Axon deployment.

1. First，it is necessary to [compile](https://github.com/axonweb3/axon?tab=readme-ov-file#compile-from-source) Axon in a local or standardized Rust environment.

2. Next, generate the corresponding key using [generate-keypair](https://github.com/axonweb3/axon/tree/main/core/cli#generate-keypair)



### 2. Axon Kubernetes Deployment

- Detailed instructions and files for deploying Axon on Kubernetes.
- Navigate to the [kubernetes-deployment](https://github.com/axonweb3/axon/tree/main/devtools/chain/k8s) directory for Kubernetes-specific deployment.

### 3. Node Deployment

- Instructions for deploying Axon on individual nodes.
- Explore the [node-deployment](https://github.com/axonweb3/axon/tree/main/devtools/chain/nodes) directory for node-specific deployment details.

## Important Note

Regardless of the chosen deployment method, it is strongly advised to generate a unique private key for added security. Follow the instructions in the respective directories to create your own key.

Feel free to explore each deployment method based on your specific requirements and preferences.
