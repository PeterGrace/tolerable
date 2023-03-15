# tolerable
## Auto-configure tolerations for architecture-based taints
This software program was written due to the author's frustration with running a mixed-architecture kubernetes cluster.  When you are running both amd64 and arm64 processors in your cluster, there is not automatic method of detecting whether an image is built for arm64 or not.  This results in Pods going into CrashLoopBackOff with a log line similar to "exec format error."

`tolerable` is a Kubernetes Mutating Webhook which watches the creation of Pods in the cluster.  When a request for a Pod is tendered, the Mutating Webhook receives a copy of the request.  It then asks the registry that houses the image whether it has an arm64 version.  If the registry responds affirmatively, it patches in a pre-configured toleration to the pod so it can schedule on the specified architecture's node(s).

## quickstart
there is a kustomize/ folder that has a kustomization spec for deploying the service.  It relies on cert-manager to create the certificates required to enable a MutatingWebhookConfiguration.  If you don't have cert-manager, you'll need to generate these certs manually and patch the containing secret into your deployment.

## configs

### tolerable.toml
| value | what it is |
| ----- | ---------- |
| ssl_key_path | path to private key, pem format |
| ssl_cert_path | path to cert, pem format |
| registry_credential_path | specified path to individual credential files, toml format |
| supported_architectures | array of valid architectures for the kubernetes cluster |
| [tolerations] | a toml object definition that contains the toleration format for the kubernetes cluster |

### tolerations
This will be specific to your deployment.  In my case, I set a taint on my arm64 nodes of `kubernetes.io/arch=arm64:NoSchedule` which means only a toleration that matches that taint will be allowed to schedule.  The toleration for this taint is here:
```
[tolerations]
effect = "NoSchedule"
operator = "Equal"
key = "kubernetes.io/arch"
```

### creds files (e.g., docker.io.toml)
Each registry that you need to auth to should be specified in a toml file, named for the registry name of the registry with .toml suffixed, and the file should contain two keys, `user` and `secret`.
Example:
```
$ cat creds/docker.io.toml
user="foobar"
secret="bazbat"
```



