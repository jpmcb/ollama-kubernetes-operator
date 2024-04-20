# 🦙 Ollama-Kubernetes-Operator

A Kubernetes operator to simplify deploying Ollama as GPU workloads to the right nodes.

_A current work in progress._

### Why?

There's no definitive way to deploy Ollama onto a subset of nodes that support
fast, GPU workloads. You can piece-meal it together, but you end up with alot of
tainted deployments with a mess of labels that's difficult to maintain.
The aim of the Ollama-Kubernetes-Operator is to enable Kubernetes practitioners
to easily deploy open source LLM technologies
as a service to their clusters with as minimal maintenance as possible.
