# Todo CLI 🦀

> Gerenciador de tarefas em linha de comando - Projeto de estudo em Rust

Um gerenciador de tarefas simples e funcional desenvolvido para aprender Rust na prática, com foco em CLI, manipulação de arquivos e tratamento de erros.

## 📚 Evolução do projeto

Este projeto foi desenvolvido de forma incremental. Cada versão adiciona uma feature e conceitos novos:

| Versão | Descrição | Conceitos |
|--------|-----------|-----------|
| [v0.1.0] | CLI básica com add/list | `OpenOptions`, `writeln!`, `enumerate`, `match`, `?` operator |
| [v0.2.0] | Comando done para marcar conclusão | `parse()`, `.map().collect()`, `.replace()`, `Vec<String>`, `.join()`, `fs::write()` |
| [v0.3.0] | Comando remove para deletar tarefas | `Vec::remove()`, validação de índices, tratamento de erros |
| [v0.4.0] | Comando undone para desmarcar | manipulação inversa de estados, lógica booleana |
| [v0.4.1] | 🐛 Correção: bug no comando list | `trim()`, filtro de linhas vazias, tratamento robusto |
| [v0.4.2] | 🐛 Correção: validações de estado | validação de duplicação, mensagens específicas, pré-condições |
| [v0.5.0] | Comando clear para limpar tudo | `fs::remove_file()`, `fs::metadata()`, tratamento completo |

[v0.1.0]: https://github.com/joaofelipegalvao/todo-cli/releases/tag/v0.1.0
[v0.2.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.1.0...v0.2.0
[v0.3.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.2.0...v0.3.0
[v0.4.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.3.0...v0.4.0
[v0.4.1]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.0...v0.4.1
[v0.4.2]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.1...v0.4.2
[v0.5.0]: https://github.com/joaofelipegalvao/todo-cli/compare/v0.4.2...v0.5.0

## 📋 Comandos disponíveis

| Comando | Descrição | Exemplo |
|---------|-----------|---------|
| `add <tarefa>` | Adiciona nova tarefa | `todo add "Estudar Rust"` |
| `list` | Lista todas as tarefas | `todo list` |
| `done <número>` | Marca tarefa como concluída | `todo done 1` |
| `undone <número>` | Desmarca tarefa | `todo undone 1` |
| `remove <número>` | Remove tarefa específica | `todo remove 1` |
| `clear` | Remove todas as tarefas | `todo clear` |

```bash
# Ver código de uma versão específica
git checkout v0.1.0  # ou qualquer tag acima
```

## 🐛 Bugs encontrados e corrigidos

Durante o desenvolvimento, alguns bugs interessantes foram descobertos e resolvidos:

### Bug #1: Linha fantasma no list

**Problema:** Ao remover todas as tarefas, `list` mostrava "1." sem conteúdo.  
**Causa:** Arquivo ficava com linha vazia após remoção.  
**Solução:** Filtro `.filter(|l| !l.trim().is_empty())` em todas as operações de leitura.  
**Versão:** v0.4.1

### Bug #2: Duplicação de marcação

**Problema:** Marcar tarefa já concluída causava `[x][x]` no arquivo, corrompendo os dados.  
**Causa:** Falta de validação de estado antes de aplicar `.replace()`.  
**Solução:** Verificação com `.contains("[x]")` antes de marcar como concluída.  
**Versão:** v0.4.2

### Bug #3: Índices incorretos após filtro

**Problema:** Números mostrados no `list` não correspondiam aos índices reais do arquivo.  
**Causa:** Linhas vazias no arquivo causavam desalinhamento entre visualização e dados.  
**Solução:** Filtrar linhas vazias consistentemente em **todos** os comandos (done, undone, remove).  
**Versão:** v0.4.2

## 🎨 Decisões de design

### Por que `clear` em vez de deletar automaticamente?

Inicialmente consideramos deletar `todos.txt` automaticamente quando a última tarefa fosse removida. Decidimos criar um comando `clear` explícito porque:

- ✅ Respeita a intenção do usuário (ação explícita vs comportamento implícito)
- ✅ Evita surpresas (usuário pode querer manter arquivo vazio)
- ✅ Mais previsível e consistente
- ✅ Permite reversão (arquivo existe até ser explicitamente removido)

### Por que validar estado antes de marcar?

Impedir que tarefas sejam marcadas múltiplas vezes:

- ✅ Evita corrupção do arquivo (`[x][x]` duplicado)
- ✅ Garante integridade dos dados
- ✅ Mensagens de erro mais claras para o usuário
- ✅ Comportamento idempotente (executar 2x = mesma coisa que 1x)

### Por que filtrar linhas vazias em todos os comandos?

Garante robustez mesmo se:

- ✅ Arquivo for editado manualmente
- ✅ Houver corrupção de dados
- ✅ Bugs gerarem linhas vazias
- ✅ Formato for inconsistente

## 💡 O que aprendi

### Manipulação de arquivos

- ✅ `OpenOptions` com `.create()` e `.append()` para adicionar sem sobrescrever
- ✅ `writeln!` macro para escrita formatada em arquivos
- ✅ `fs::read_to_string()` para leitura completa
- ✅ `fs::write()` para sobrescrever arquivo inteiro
- ✅ `fs::remove_file()` para deletar arquivos
- ✅ `fs::metadata()` para verificar existência sem abrir

### Strings e coleções

- ✅ `enumerate()` para obter índices + valores em loops
- ✅ `parse()` para conversão string → número com validação
- ✅ `.map().collect()` para transformar iteradores em coleções
- ✅ `.replace()` para substituição de texto
- ✅ `.contains()` para busca em strings
- ✅ `.trim()` para remover espaços em branco
- ✅ `.join()` para concatenar com separador
- ✅ `.filter()` para selecionar elementos
- ✅ `Vec::remove()` para deletar elementos por índice

### Controle de fluxo e erros

- ✅ Pattern matching com `match` para subcomandos
- ✅ Tratamento de erros com `?` operator (propagação automática)
- ✅ `Result<T, E>` para funções que podem falhar
- ✅ `Box<dyn Error>` para erros genéricos
- ✅ `if let` para pattern matching simplificado
- ✅ Validação de entrada e pré-condições

### CLI e UX

- ✅ `env::args()` para capturar argumentos da linha de comando
- ✅ Subcomandos com pattern matching
- ✅ Validação de entrada (argumentos, números, estados)
- ✅ `println!` vs `eprintln!` (stdout vs stderr)
- ✅ `process::exit()` para códigos de saída
- ✅ Mensagens de erro específicas e úteis

### Debug e qualidade

- ✅ Encontrar e corrigir bugs através de testes manuais
- ✅ Validação de pré-condições (evitar estados inválidos)
- ✅ Pensamento em edge cases (arquivo vazio, índices inválidos)
- ✅ Uso de debug prints (`eprintln!`) para investigação
- ✅ Refatoração iterativa (melhorar sem quebrar)

## 📦 Instalação

```bash
# Clonar repositório
git clone https://github.com/joaofelipegalvao/todo-cli
cd todo-cli

# Compilar
cargo build --release

# Instalar globalmente (opcional)
sudo cp target/release/todo-cli /usr/local/bin/todo
```

## 🚀 Como usar

### Após instalar globalmente

```bash
todo add "Estudar Rust"
todo list
todo done 1
todo undone 1
todo remove 1
todo clear
```

### Com Cargo (desenvolvimento)

```bash
cargo run -- add "Estudar Rust"
cargo run -- list
cargo run -- done 1
cargo run -- undone 1
cargo run -- remove 1
cargo run -- clear
```

## 🎯 Roadmap

### Implementado ✅

- [x] Comando add para adicionar tarefas
- [x] Comando list para listar todas
- [x] Comando done para marcar como concluída
- [x] Comando undone para desmarcar
- [x] Comando remove para deletar específica
- [x] Comando clear para limpar todas
- [x] Validação completa de erros
- [x] Tratamento robusto de arquivo

### Próximos passos 🔮

- [ ] Testes unitários
- [ ] Cores no terminal (tarefas concluídas em verde)
- [ ] Contador de progresso ("2 de 5 concluídas")
- [ ] Prioridades (alta/média/baixa)
- [ ] Categorias/tags (#trabalho, #casa)
- [ ] Busca e filtros
- [ ] Data de criação/vencimento
- [ ] Formato JSON para dados estruturados

---

**Projeto desenvolvido como parte do aprendizado de Rust** 🦀  
*Cada commit representa um passo no processo de aprendizado*
