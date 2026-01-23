Gerenciador de tarefas em linha de comando - Projeto de estudo em Rust 🦀

## 📝 Todo CLI

Este projeto foi desenvolvido em 1 etapa, marcada por tag:

| Versão | Descrição | Conceitos |
|--------|-----------|-----------|
| [v1] | CLI básica com add/list | `OpenOptions`, `writeln!`, `enumerate`, `match`, `?` operator |

[v1]: https://github.com/joaofelipegalvao/todo-cli/releases/tag/v0.1.0

## 🚀 Como usar

```bash
# Adicionar tarefa
cargo run -- add "Estudar Rust"

# Listar tarefas
cargo run -- list

# Ver versão específica
git checkout v0.1.0  # ou qualquer tag
```

## 📋 Comandos disponíveis

| Comando | Descrição | Exemplo |
|---------|-----------|---------|
| add | Adiciona nova tarefa | todo add "Estudar Rust" |
| list | Lista todas as tarefas | todo list |

## 💡 O que aprendi

- ✅ Manipulação de arquivos com OpenOptions
- ✅ Escrita com writeln! macro
- ✅ Enumeração com enumerate()
- ✅ Pattern matching com match
- ✅ Tratamento de erros com ? operator
- ✅ CLI argument parsing

## 🎯 Próximos passos

- [ ] Comando remove para deletar tarefas
- [ ] Comando done para marcar como concluída
- [ ] Persistência de estado (concluídas/pendentes)
- [ ] Testes unitários

---

Nota: Este é um projeto de aprendizado. Cada tag representa um passo evolutivo.
