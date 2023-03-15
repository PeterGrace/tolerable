# tolerable
## Auto-configure tolerations for architecture-based taints
This software program was written due to the author's frustration with running a mixed-architecture kubernetes cluster.  When you are running both amd64 and arm64 processors in your cluster, there is not automatic method of detecting whether an image is built for arm64 or not.  This results in Pods going into CrashLoopBackOff with a log line similar to "exec format error."

`tolerable` is a Kubernetes Mutating Webhook which watches the creation of Pods in the cluster.  When a request for a Pod is tendered, the Mutating Webhook receives a copy of the request.  It then asks the registry that houses the image whether it has an arm64 version.  If the registry responds affirmatively, it patches in a pre-configured toleration to the pod so it can schedule on the specified architecture's node(s).

## quickstart
there is a kustomize/ folder that has a kustomization spec for deploying the service.  It relies on cert-manager to create the certificates required to enable a MutatingWebhookConfiguration.  If you don't have cert-manager, you'll need to generate these certs manually and patch the containing secret into your deployment.
