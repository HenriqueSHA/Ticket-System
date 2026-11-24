# 8. Algoritmo proposto — Sistema de Cadastro e Venda de Ingressos (Equipe 11)

## 8.1 Estratégia utilizada

A solução segue a decomposição feita na Entrega 2: o problema geral (centralizar o cadastro e a venda
de ingressos) foi dividido em **módulos**, e cada módulo virou um **sub-algoritmo** independente,
chamado a partir de um programa principal.

- O **programa principal** fica em um laço de repetição exibindo o menu inicial até o usuário escolher sair.
- Após o login, uma **estrutura de decisão** sobre o perfil do usuário (cliente, organizador ou SuperAdmin)
  libera apenas o menu correspondente (controle de acesso — seção 7.4).
- Toda entrada de dados passa por **validação antes de qualquer gravação** (regra 7.3). Quando o dado é
  inválido, o algoritmo exibe uma mensagem e pede novamente (laço `ENQUANTO`) ou cancela a operação.
- A **venda duplicada** é evitada verificando a situação do assento e a quantidade disponível da categoria
  imediatamente antes de registrar a venda; a gravação da venda, a ocupação do assento e a baixa na
  quantidade disponível acontecem juntas, como uma única operação.
- Os assentos de cada setor são organizados como uma **matriz** (fileira × número), percorrida com dois
  laços `PARA` aninhados para gerar e exibir o mapa.
- Na implementação, o "menu" de cada perfil é a barra de navegação da interface web e cada opção é uma
  página. O laço do programa principal corresponde ao servidor, que fica atendendo pedidos até ser
  encerrado. A lógica (decisões, repetições e validações) é a mesma do pseudocódigo, e cada algoritmo
  tem um método correspondente no código (tabela da seção 12).

### Convenções do pseudocódigo

| Notação | Significado |
|---|---|
| `←` | atribuição |
| `LEIA` / `ESCREVA` | entrada / saída de dados |
| `SE … ENTÃO … SENÃO … FIMSE` | estrutura condicional |
| `ESCOLHA … CASO … OUTROCASO … FIMESCOLHA` | condicional de múltipla escolha (menus) |
| `ENQUANTO … FAÇA … FIMENQUANTO` | repetição com teste no início |
| `PARA i DE a ATÉ b FAÇA … FIMPARA` | repetição com contador |
| `PROCEDIMENTO` / `FUNÇÃO … RETORNE` | sub-algoritmo sem / com valor de retorno |
| `NULO` | registro não encontrado |
| `HOJE()` | data atual do sistema |

### Constantes

```
CONSTANTE DESCONTO_MEIA  ← 50        // meia-entrada: 50% de desconto (percentual)
CONSTANTE LIVRE          ← "LIVRE"
CONSTANTE OCUPADO        ← "OCUPADO"
```

---

## 8.2 Algoritmo 1 — Programa principal (login e menu por perfil)

```
ALGORITMO "SistemaIngressos"
DECLARE opcao : inteiro
        email, senha : texto
        usuario : Usuario
        sair : lógico
INÍCIO
    CARREGAR dados do banco de dados
    sair ← FALSO
    ENQUANTO sair = FALSO FAÇA
        ESCREVA "1 - Entrar   2 - Criar conta de cliente   3 - Criar conta de organizador   0 - Sair"
        LEIA opcao
        ESCOLHA opcao
            CASO 1:
                ESCREVA "E-mail: "
                LEIA email
                ESCREVA "Senha: "
                LEIA senha
                usuario ← buscarUsuarioPorEmail(email)
                SE usuario = NULO OU gerarHash(senha) <> usuario.senhaHash ENTÃO
                    ESCREVA "E-mail ou senha inválidos."
                SENÃO
                    ESCOLHA usuario.perfil
                        CASO "CLIENTE":
                            menuCliente(usuario)
                        CASO "ORGANIZADOR":
                            SE organizadorDoUsuario(usuario).status = "BLOQUEADO" ENTÃO
                                ESCREVA "Acesso bloqueado. Procure o administrador."
                            SENÃO
                                menuOrganizador(usuario)
                            FIMSE
                        CASO "SUPERADMIN":
                            menuSuperAdmin(usuario)
                    FIMESCOLHA
                FIMSE
            CASO 2:
                cadastrarCliente()
            CASO 3:
                cadastrarOrganizador()          // Algoritmo 9
            CASO 0:
                sair ← VERDADEIRO
            OUTROCASO:
                ESCREVA "Opção inválida."
        FIMESCOLHA
    FIMENQUANTO
    ESCREVA "Sistema encerrado."
FIM
```

**Menus por perfil** (cada opção chama um sub-algoritmo; todos ficam em laço até a opção 0 — Voltar):

| Cliente | Organizador | SuperAdmin |
|---|---|---|
| 1 - Listar eventos disponíveis | 1 - Cadastrar evento (Alg. 3) | 1 - Cadastrar organizador |
| 2 - Comprar ingresso (Alg. 4) | 2 - Gerenciar evento: dados, categorias, preço, quantidade e capacidade (Alg. 8) | 2 - Listar organizadores |
| 3 - Meus ingressos / histórico | 3 - Listar meus eventos | 3 - Ativar / bloquear / editar / remover organizador (Alg. 7) |
| 4 - Cancelar compra (Alg. 5) | 4 - Relatório do evento e lista de vendas (Alg. 6) | 0 - Sair |
| 5 - Trocar assento (Alg. 5b) | 5 - Registrar presença (check-in, Alg. 6) | |
| 0 - Sair | 0 - Sair | |

---

## 8.3 Algoritmo 2 — Cadastrar cliente (com validação de CPF)

```
PROCEDIMENTO cadastrarCliente()
DECLARE nome, cpf, email, senha : texto
INÍCIO
    ESCREVA "Nome: "
    LEIA nome
    ESCREVA "CPF (somente números): "
    LEIA cpf
    ESCREVA "E-mail: "
    LEIA email
    ESCREVA "Senha (mínimo 6 caracteres): "
    LEIA senha

    SE nome = "" OU cpf = "" OU email = "" OU senha = "" ENTÃO
        ESCREVA "Erro: todos os campos são obrigatórios."
    SENÃO SE validarCPF(cpf) = FALSO ENTÃO
        ESCREVA "Erro: CPF inválido."
    SENÃO SE email não contém "@" ENTÃO
        ESCREVA "Erro: e-mail inválido."
    SENÃO SE tamanho(senha) < 6 ENTÃO
        ESCREVA "Erro: a senha deve ter pelo menos 6 caracteres."
    SENÃO SE buscarUsuarioPorEmail(email) <> NULO OU buscarUsuarioPorDocumento(cpf) <> NULO ENTÃO
        ESCREVA "Erro: já existe um usuário com este e-mail ou CPF."
    SENÃO
        GRAVAR Usuario(nome, cpf, email, gerarHash(senha), perfil ← "CLIENTE")
        ESCREVA "Cadastro realizado com sucesso!"
    FIMSE
FIM
```

### Função auxiliar — validar CPF (dígitos verificadores)

O CPF tem 11 dígitos; os dois últimos são calculados a partir dos 9 primeiros.

```
FUNÇÃO validarCPF(cpf : texto) : lógico
DECLARE d : vetor[1..11] de inteiro
        i, soma, resto, dv1, dv2 : inteiro
        todosIguais : lógico
INÍCIO
    SE tamanho(cpf) <> 11 OU cpf contém caractere que não é dígito ENTÃO
        RETORNE FALSO
    FIMSE
    PARA i DE 1 ATÉ 11 FAÇA
        d[i] ← converterParaInteiro(cpf[i])
    FIMPARA

    // CPFs com todos os dígitos iguais (ex.: 111.111.111-11) são inválidos
    todosIguais ← VERDADEIRO
    PARA i DE 2 ATÉ 11 FAÇA
        SE d[i] <> d[1] ENTÃO
            todosIguais ← FALSO
        FIMSE
    FIMPARA
    SE todosIguais ENTÃO
        RETORNE FALSO
    FIMSE

    // 1º dígito verificador: pesos 10, 9, ..., 2
    soma ← 0
    PARA i DE 1 ATÉ 9 FAÇA
        soma ← soma + d[i] * (11 - i)
    FIMPARA
    resto ← (soma * 10) % 11
    SE resto = 10 ENTÃO resto ← 0 FIMSE
    dv1 ← resto

    // 2º dígito verificador: pesos 11, 10, ..., 2
    soma ← 0
    PARA i DE 1 ATÉ 10 FAÇA
        soma ← soma + d[i] * (12 - i)
    FIMPARA
    resto ← (soma * 10) % 11
    SE resto = 10 ENTÃO resto ← 0 FIMSE
    dv2 ← resto

    RETORNE (dv1 = d[10]) E (dv2 = d[11])
FIM
```

---

## 8.4 Algoritmo 3 — Cadastrar evento (organizador)

O organizador informa os dados do evento e, em seguida, as categorias de ingresso. Categorias com
assento marcado (ex.: Camarote, Cadeira) viram um **setor** com uma matriz de assentos; categorias sem
assento (ex.: Pista) têm apenas uma quantidade.

```
PROCEDIMENTO cadastrarEvento(organizador : Organizador)
DECLARE nome, descricao, local, resposta, nomeCategoria : texto
        data : data
        horario : hora
        capacidade, quantidade, fileiras, assentosPorFileira, totalCategorias : inteiro
        preco : real
        permiteCancelamento, comAssento : lógico
        evento : Evento
        categoria : Ingresso
INÍCIO
    SE organizador.status <> "ATIVO" ENTÃO
        ESCREVA "Erro: apenas organizadores ativos podem cadastrar eventos."
        RETORNE
    FIMSE

    ESCREVA "Nome, descrição, local, data (dd/mm/aaaa), horário (hh:mm), capacidade total: "
    LEIA nome, descricao, local, data, horario, capacidade
    ESCREVA "Permite cancelamento pelo cliente? (S/N): "
    LEIA resposta
    permiteCancelamento ← (resposta = "S")

    SE nome = "" OU local = "" ENTÃO
        ESCREVA "Erro: nome e local são obrigatórios."
        RETORNE
    FIMSE
    SE data <= HOJE() ENTÃO
        ESCREVA "Erro: a data do evento deve ser futura."
        RETORNE
    FIMSE
    SE capacidade <= 0 ENTÃO
        ESCREVA "Erro: a capacidade deve ser maior que zero."
        RETORNE
    FIMSE

    evento ← NOVO Evento(nome, descricao, local, data, horario, capacidade,
                         permiteCancelamento, status ← "ABERTO", organizador.id)

    // ---- Cadastro das categorias de ingresso ----
    totalCategorias ← 0
    resposta ← "S"
    ENQUANTO resposta = "S" FAÇA
        ESCREVA "Nome da categoria (ex.: Pista, VIP, Camarote): "
        LEIA nomeCategoria
        ESCREVA "Preço (R$): "
        LEIA preco
        ESCREVA "Possui assento marcado? (S/N): "
        LEIA resposta
        comAssento ← (resposta = "S")

        SE comAssento ENTÃO
            ESCREVA "Quantidade de fileiras e de assentos por fileira: "
            LEIA fileiras, assentosPorFileira
            quantidade ← fileiras * assentosPorFileira
        SENÃO
            ESCREVA "Quantidade de ingressos: "
            LEIA quantidade
        FIMSE

        SE preco < 0 ENTÃO
            ESCREVA "Erro: o preço não pode ser negativo. Categoria ignorada."
        SENÃO SE quantidade <= 0 ENTÃO
            ESCREVA "Erro: a quantidade deve ser maior que zero. Categoria ignorada."
        SENÃO SE totalCategorias + quantidade > capacidade ENTÃO
            ESCREVA "Erro: a soma das categorias ultrapassa a capacidade do evento (",
                    capacidade - totalCategorias, " vagas restantes)."
        SENÃO
            categoria ← NOVO Ingresso(nomeCategoria, preco, quantidade, comAssento)
            ADICIONAR categoria À LISTA evento.categorias
            totalCategorias ← totalCategorias + quantidade
            SE comAssento ENTÃO
                gerarAssentos(evento, categoria, fileiras, assentosPorFileira)
            FIMSE
        FIMSE

        ESCREVA "Cadastrar outra categoria? (S/N): "
        LEIA resposta
    FIMENQUANTO

    SE evento.categorias está vazia ENTÃO
        ESCREVA "Erro: o evento precisa de pelo menos uma categoria. Cadastro cancelado."
    SENÃO
        GRAVAR evento, suas categorias e seus assentos
        ESCREVA "Evento cadastrado com sucesso! Código: ", evento.id
    FIMSE
FIM
```

### Sub-algoritmo — gerar a matriz de assentos

```
PROCEDIMENTO gerarAssentos(evento, categoria, fileiras, assentosPorFileira : inteiro)
DECLARE f, n : inteiro
INÍCIO
    PARA f DE 1 ATÉ fileiras FAÇA
        PARA n DE 1 ATÉ assentosPorFileira FAÇA
            // fileira 1 → "A", 2 → "B", ...
            ADICIONAR Assento(setor ← categoria.nome, fileira ← letra(f), numero ← n,
                              situacao ← LIVRE, categoria.id, evento.id)
                      À LISTA evento.assentos
        FIMPARA
    FIMPARA
FIM
```

### Sub-algoritmo — exibir o mapa de assentos

```
PROCEDIMENTO exibirMapaAssentos(categoria : Ingresso)
DECLARE f, n : inteiro
INÍCIO
    ESCREVA "Setor: ", categoria.nome, "   [ ] livre   [X] ocupado"
    PARA f DE 1 ATÉ categoria.fileiras FAÇA
        ESCREVA letra(f), "  "
        PARA n DE 1 ATÉ categoria.assentosPorFileira FAÇA
            SE assento(categoria, f, n).situacao = OCUPADO ENTÃO
                ESCREVA "[X]"
            SENÃO
                ESCREVA "[ ]"
            FIMSE
        FIMPARA
        ESCREVAL                       // quebra de linha
    FIMPARA
FIM
```

Exemplo de saída (setor com 3 fileiras × 5 assentos):

```
Setor: Camarote   [ ] livre   [X] ocupado
     1  2  3  4  5
A  [ ][X][ ][ ][ ]
B  [ ][ ][X][X][ ]
C  [ ][ ][ ][ ][ ]
```

---

## 8.5 Algoritmo 4 — Comprar ingresso (cliente)

Fluxo central do sistema. Cada verificação que falha encerra a compra com uma mensagem, sem gravar nada.

```
PROCEDIMENTO comprarIngresso(cliente : Usuario)
DECLARE idEvento, idCategoria, tipo, numero, formaPag : inteiro
        fileira, confirma : texto
        valor : real
        evento : Evento
        categoria : Ingresso
        assento : Assento
        venda : Venda
INÍCIO
    listarEventosDisponiveis()     // status "ABERTO", data futura e com ingressos disponíveis
    ESCREVA "Código do evento: "
    LEIA idEvento
    evento ← buscarEvento(idEvento)
    SE evento = NULO OU evento.status <> "ABERTO" OU evento.data <= HOJE() ENTÃO
        ESCREVA "Erro: evento inexistente ou indisponível para venda."
        RETORNE
    FIMSE
    SE evento.totalVendido >= evento.capacidade ENTÃO
        ESCREVA "Evento lotado."
        RETORNE
    FIMSE

    // ---- Escolha da categoria ----
    PARA CADA c EM evento.categorias FAÇA
        ESCREVA c.id, " - ", c.nome, "  R$ ", c.preco, "  (", c.disponivel, " disponíveis)"
    FIMPARA
    ESCREVA "Código da categoria: "
    LEIA idCategoria
    categoria ← buscarCategoria(evento, idCategoria)
    SE categoria = NULO ENTÃO
        ESCREVA "Erro: categoria inexistente neste evento."
        RETORNE
    FIMSE
    SE categoria.disponivel = 0 ENTÃO
        ESCREVA "Categoria esgotada."
        RETORNE
    FIMSE

    // ---- Tipo de ingresso (inteira / meia) ----
    ESCREVA "1 - Inteira   2 - Meia-entrada"
    LEIA tipo
    ENQUANTO tipo <> 1 E tipo <> 2 FAÇA
        ESCREVA "Opção inválida. Digite 1 ou 2: "
        LEIA tipo
    FIMENQUANTO

    // ---- Escolha do assento (apenas se a categoria tiver assento marcado) ----
    assento ← NULO
    SE categoria.comAssento ENTÃO
        exibirMapaAssentos(categoria)
        ESCREVA "Fileira (letra) e número do assento: "
        LEIA fileira, numero
        assento ← buscarAssento(categoria, fileira, numero)
        SE assento = NULO ENTÃO
            ESCREVA "Erro: assento inexistente."
            RETORNE
        FIMSE
        SE assento.situacao = OCUPADO ENTÃO
            ESCREVA "Erro: assento já vendido. Escolha outro."
            RETORNE
        FIMSE
    FIMSE

    // ---- Cálculo do valor ----
    valor ← categoria.preco
    SE tipo = 2 ENTÃO
        valor ← valor * (1 - DESCONTO_MEIA)
    FIMSE

    // ---- Forma de pagamento ----
    ESCREVA "Forma de pagamento: 1 - PIX   2 - Cartão de crédito   3 - Cartão de débito"
    LEIA formaPag
    ENQUANTO formaPag < 1 OU formaPag > 3 FAÇA
        ESCREVA "Opção inválida: "
        LEIA formaPag
    FIMENQUANTO

    // ---- Confirmação ----
    ESCREVA "Evento: ", evento.nome, " | Categoria: ", categoria.nome,
            " | Assento: ", fileira, numero, " | Total: R$ ", valor
    ESCREVA "Confirmar compra? (S/N): "
    LEIA confirma
    SE confirma <> "S" ENTÃO
        ESCREVA "Compra cancelada pelo usuário."
        RETORNE
    FIMSE

    // ---- Registro da venda (operação única: tudo ou nada) ----
    INICIAR TRANSAÇÃO
        SE assento <> NULO ENTÃO
            assento.situacao ← OCUPADO
        FIMSE
        categoria.disponivel ← categoria.disponivel - 1
        evento.totalVendido ← evento.totalVendido + 1
        venda ← NOVA Venda(HOJE(), valor, tipo, formaPag, status ← "CONFIRMADA",
                           cliente.id, categoria.id, assento)   // assento pode ser NULO (ex.: Pista)
        GRAVAR venda
    CONFIRMAR TRANSAÇÃO

    ESCREVA "Compra realizada! Comprovante nº ", venda.id
    exibirComprovante(venda)
FIM
```

---

## 8.6 Algoritmo 5 — Cancelar compra (cliente)

```
PROCEDIMENTO cancelarCompra(cliente : Usuario)
DECLARE idVenda : inteiro
        confirma : texto
        venda : Venda
        evento : Evento
INÍCIO
    listarVendas(cliente, status ← "CONFIRMADA")
    ESCREVA "Número da compra a cancelar: "
    LEIA idVenda
    venda ← buscarVenda(idVenda)

    SE venda = NULO OU venda.idCliente <> cliente.id ENTÃO
        ESCREVA "Erro: compra não encontrada."
        RETORNE
    FIMSE
    SE venda.status <> "CONFIRMADA" ENTÃO
        ESCREVA "Erro: esta compra já foi cancelada ou utilizada."
        RETORNE
    FIMSE
    evento ← eventoDaVenda(venda)
    SE evento.permiteCancelamento = FALSO ENTÃO
        ESCREVA "Erro: este evento não permite cancelamento."
        RETORNE
    FIMSE
    SE evento.data <= HOJE() ENTÃO
        ESCREVA "Erro: não é possível cancelar após a data do evento."
        RETORNE
    FIMSE

    ESCREVA "Confirmar cancelamento? Valor a estornar: R$ ", venda.valor, " (S/N): "
    LEIA confirma
    SE confirma = "S" ENTÃO
        INICIAR TRANSAÇÃO
            venda.status ← "CANCELADA"
            SE venda.idAssento <> NULO ENTÃO
                assentoDaVenda(venda).situacao ← LIVRE
            FIMSE
            categoriaDaVenda(venda).disponivel ← categoriaDaVenda(venda).disponivel + 1
            evento.totalVendido ← evento.totalVendido - 1
        CONFIRMAR TRANSAÇÃO
        ESCREVA "Compra cancelada. O assento foi liberado."
    FIMSE
FIM
```

---

## 8.6b Algoritmo 5b — Remarcar compra (trocar assento)

A remarcação segue as mesmas regras do cancelamento (regra 7.3): a compra é do cliente, está
confirmada, o evento permite alterações e ainda não ocorreu. O novo assento precisa ser da **mesma
categoria** e estar livre. O valor pago não muda.

```
PROCEDIMENTO remarcarCompra(cliente : Usuario)
DECLARE idVenda, numero : inteiro
        fileira : texto
        venda : Venda
        evento : Evento
        novo : Assento
INÍCIO
    listarVendas(cliente, status ← "CONFIRMADA")
    ESCREVA "Número da compra a remarcar: "
    LEIA idVenda
    venda ← buscarVenda(idVenda)

    SE venda = NULO OU venda.idCliente <> cliente.id ENTÃO
        ESCREVA "Erro: compra não encontrada."
        RETORNE
    FIMSE
    SE venda.status <> "CONFIRMADA" ENTÃO
        ESCREVA "Erro: esta compra já foi cancelada ou utilizada."
        RETORNE
    FIMSE
    evento ← eventoDaVenda(venda)
    SE evento.permiteCancelamento = FALSO OU evento.data <= HOJE() ENTÃO
        ESCREVA "Erro: este evento não permite remarcar a compra."
        RETORNE
    FIMSE
    SE venda.idAssento = NULO ENTÃO
        ESCREVA "Erro: esta compra não tem assento marcado para trocar."
        RETORNE
    FIMSE

    exibirMapa(categoriaDaVenda(venda))
    ESCREVA "Nova fileira e número: "
    LEIA fileira, numero
    novo ← buscarAssento(categoriaDaVenda(venda), fileira, numero)
    SE novo = NULO ENTÃO
        ESCREVA "Erro: assento inexistente."
    SENÃO SE novo.situacao = OCUPADO ENTÃO
        ESCREVA "Erro: assento já vendido. Escolha outro."
    SENÃO
        INICIAR TRANSAÇÃO
            novo.situacao ← OCUPADO
            assentoDaVenda(venda).situacao ← LIVRE
            venda.idAssento ← novo.id
        CONFIRMAR TRANSAÇÃO
        ESCREVA "Assento trocado para ", fileira, numero
    FIMSE
FIM
```

---

## 8.7 Algoritmo 6 — Relatório do evento e registro de presença (organizador)

```
PROCEDIMENTO relatorioEvento(organizador : Organizador)
DECLARE idEvento, vendidos, cancelados, presentes, inteiras, meias : inteiro
        receita, ocupacao : real
        evento : Evento
INÍCIO
    ESCREVA "Código do evento: "
    LEIA idEvento
    evento ← buscarEvento(idEvento)
    SE evento = NULO OU evento.idOrganizador <> organizador.id ENTÃO
        ESCREVA "Erro: evento não encontrado entre os seus eventos."
        RETORNE
    FIMSE

    vendidos ← 0   cancelados ← 0   presentes ← 0
    inteiras ← 0   meias ← 0        receita ← 0

    PARA CADA v EM vendasDoEvento(evento) FAÇA
        SE v.status = "CANCELADA" ENTÃO
            cancelados ← cancelados + 1
        SENÃO
            vendidos ← vendidos + 1
            receita ← receita + v.valor
            SE v.tipo = 1 ENTÃO inteiras ← inteiras + 1 SENÃO meias ← meias + 1 FIMSE
            SE v.status = "UTILIZADA" ENTÃO
                presentes ← presentes + 1
            FIMSE
        FIMSE
    FIMPARA

    ocupacao ← (vendidos / evento.capacidade) * 100

    ESCREVA "Ingressos vendidos: ", vendidos, " (inteiras: ", inteiras, ", meias: ", meias, ")"
    ESCREVA "Cancelados: ", cancelados
    ESCREVA "Ocupação: ", ocupacao:0:1, "%  (", vendidos, " de ", evento.capacidade, ")"
    ESCREVA "Presentes (check-in): ", presentes, " de ", vendidos
    ESCREVA "Receita total: R$ ", receita:0:2
    PARA CADA c EM evento.categorias FAÇA
        ESCREVA c.nome, ": ", c.quantidade - c.disponivel, " vendidos de ", c.quantidade
        SE c.comAssento ENTÃO
            exibirMapaAssentos(c)
        FIMSE
    FIMPARA
FIM
```

```
PROCEDIMENTO registrarPresenca(organizador : Organizador)
DECLARE idVenda : inteiro
        venda : Venda
INÍCIO
    ESCREVA "Número do ingresso apresentado na entrada: "
    LEIA idVenda
    venda ← buscarVenda(idVenda)
    SE venda = NULO OU eventoDaVenda(venda).idOrganizador <> organizador.id ENTÃO
        ESCREVA "Erro: ingresso não encontrado para os seus eventos."
    SENÃO SE venda.status = "CANCELADA" ENTÃO
        ESCREVA "Acesso negado: ingresso cancelado."
    SENÃO SE venda.status = "UTILIZADA" ENTÃO
        ESCREVA "Acesso negado: ingresso já utilizado."
    SENÃO
        venda.status ← "UTILIZADA"
        GRAVAR venda
        ESCREVA "Entrada liberada."
    FIMSE
FIM
```

---

## 8.8 Algoritmo 7 — Gerenciar organizadores (SuperAdmin)

```
PROCEDIMENTO gerenciarOrganizador()
DECLARE idOrg, acao : inteiro
        razaoSocial, contato : texto
        org : Organizador
INÍCIO
    PARA CADA o EM listaDeOrganizadores FAÇA
        ESCREVA o.id, " - ", o.razaoSocial, " (", o.cnpj, ") - ", o.status
    FIMPARA
    ESCREVA "Código do organizador: "
    LEIA idOrg
    org ← buscarOrganizador(idOrg)
    SE org = NULO ENTÃO
        ESCREVA "Erro: organizador não encontrado."
        RETORNE
    FIMSE

    ESCREVA "1 - Ativar   2 - Bloquear   3 - Editar dados   4 - Remover   0 - Voltar"
    LEIA acao
    ESCOLHA acao
        CASO 1:
            org.status ← "ATIVO"
            GRAVAR org
            ESCREVA "Organizador ativado."
        CASO 2:
            org.status ← "BLOQUEADO"
            GRAVAR org
            ESCREVA "Organizador bloqueado. Ele não poderá mais acessar o sistema."
        CASO 3:
            ESCREVA "Nova razão social e novo contato: "
            LEIA razaoSocial, contato
            SE razaoSocial = "" OU contato = "" ENTÃO
                ESCREVA "Erro: campos obrigatórios."
            SENÃO
                org.razaoSocial ← razaoSocial
                org.contato ← contato
                GRAVAR org
                ESCREVA "Dados atualizados."
            FIMSE
        CASO 4:
            SE org possui eventos com vendas ENTÃO
                ESCREVA "Erro: organizador possui eventos com vendas. Bloqueie em vez de remover."
            SENÃO
                REMOVER org e seus eventos sem vendas
                ESCREVA "Organizador removido."
            FIMSE
        CASO 0:
            // volta ao menu
        OUTROCASO:
            ESCREVA "Opção inválida."
    FIMESCOLHA
FIM
```

---

## 8.9 Algoritmo 8 — Gerenciar evento (organizador)

Depois de cadastrado (Alg. 3), o evento pode ser ajustado pelo organizador dono dele. As regras
garantem que nenhuma alteração contradiga o que já foi vendido.

```
PROCEDIMENTO gerenciarEvento(org : Organizador)
DECLARE idEvento, acao, novaQtd, novaCapacidade : inteiro
        novoPreco : real
        evento : Evento
        cat : Categoria
        somaCategorias : inteiro
INÍCIO
    evento ← buscarEvento(idEvento)
    SE evento = NULO OU evento.idOrganizador <> org.id ENTÃO
        ESCREVA "Erro: evento não encontrado."
        RETORNE
    FIMSE
    ESCREVA "1 - Alterar dados  2 - Nova categoria  3 - Alterar categoria  4 - Remover categoria  5 - Excluir evento"
    LEIA acao
    ESCOLHA acao
        CASO 1:
            LEIA nome, local, data, horario, novaCapacidade, status, permiteCancelamento
            somaCategorias ← 0
            PARA CADA c EM categoriasDoEvento(evento) FAÇA
                somaCategorias ← somaCategorias + c.quantidade
            FIMPARA
            SE data <= HOJE() ENTÃO
                ESCREVA "Erro: a data deve ser futura."
            SENÃO SE novaCapacidade < evento.totalVendido OU novaCapacidade < somaCategorias ENTÃO
                ESCREVA "Erro: capacidade menor que o já vendido ou que a soma das categorias."
            SENÃO
                GRAVAR evento com os novos dados
            FIMSE
        CASO 2:
            LEIA cat                                   // nome, preço, quantidade ou fileiras × assentos
            SE cat.preco < 0 OU cat.quantidade <= 0 ENTÃO
                ESCREVA "Erro: preço e quantidade inválidos."
            SENÃO SE somaQuantidades(evento) + cat.quantidade > evento.capacidade ENTÃO
                ESCREVA "Erro: a soma das categorias ultrapassa a capacidade."
            SENÃO
                GRAVAR cat; SE cat tem assento marcado ENTÃO gerarMatrizAssentos(cat) FIMSE
            FIMSE
        CASO 3:
            LEIA cat, novoPreco, novaQtd
            vendidos ← cat.quantidade - cat.disponivel
            SE novoPreco < 0 ENTÃO
                ESCREVA "Erro: preço negativo."
            SENÃO SE novaQtd < vendidos ENTÃO
                ESCREVA "Erro: a quantidade não pode ficar abaixo do já vendido (", vendidos, ")."
            SENÃO
                cat.preco ← novoPreco                 // vale para as próximas vendas
                cat.disponivel ← novaQtd - vendidos
                cat.quantidade ← novaQtd
                GRAVAR cat
            FIMSE
        CASO 4:
            SE cat possui vendas ENTÃO ESCREVA "Erro: categoria com vendas." SENÃO REMOVER cat FIMSE
        CASO 5:
            SE evento possui vendas ENTÃO ESCREVA "Erro: evento com vendas; encerre-o." SENÃO REMOVER evento FIMSE
    FIMESCOLHA
FIM
```

---

## 8.10 Algoritmo 9 — Criar conta de organizador (autocadastro)

Na Entrega 2 só o SuperAdmin criava organizadores. Na implementação, a própria empresa pode criar a
conta pela página inicial. O SuperAdmin continua podendo cadastrar, bloquear ou remover (Alg. 7), e
um organizador bloqueado não entra no sistema (Alg. 1).

```
PROCEDIMENTO cadastrarOrganizador()
DECLARE razaoSocial, cnpj, contato, email, senha : texto
INÍCIO
    LEIA razaoSocial, cnpj, contato, email, senha
    SE razaoSocial = "" OU contato = "" ENTÃO
        ESCREVA "Erro: campos obrigatórios."
    SENÃO SE validarCNPJ(cnpj) = FALSO ENTÃO           // aceita o CNPJ numérico e o alfanumérico
        ESCREVA "Erro: CNPJ inválido."
    SENÃO SE validarEmail(email) = FALSO OU tamanho(senha) < 6 ENTÃO
        ESCREVA "Erro: e-mail inválido ou senha com menos de 6 caracteres."
    SENÃO SE existeUsuarioComEmail(email) OU existeOrganizadorComCNPJ(cnpj) ENTÃO
        ESCREVA "Erro: já existe cadastro com este e-mail ou CNPJ."
    SENÃO
        INICIAR TRANSAÇÃO
            GRAVAR Usuario(perfil ← "ORGANIZADOR", senhaHash ← gerarHash(senha))
            GRAVAR Organizador(razaoSocial, cnpj, contato, status ← "ATIVO")
        CONFIRMAR TRANSAÇÃO
        ESCREVA "Conta criada! Entre com seu e-mail e senha."
    FIMSE
FIM
```

---

## 8.11 Relação entre os algoritmos e as estruturas estudadas

| Estrutura | Onde aparece |
|---|---|
| Sequencial | Leitura dos dados e cálculo do valor na compra (Alg. 4) |
| Condicional simples/composta (`SE`) | Todas as validações das regras 7.3 (Alg. 2 a 9) |
| Múltipla escolha (`ESCOLHA`) | Menus e direcionamento por perfil (Alg. 1, 7, 8) |
| Repetição `ENQUANTO` | Laço do menu principal (Alg. 1), cadastro de várias categorias (Alg. 3), revalidação de entradas (Alg. 4) |
| Repetição `PARA` | Validação do CPF (Alg. 2), geração e exibição da matriz de assentos (Alg. 3), contadores do relatório (Alg. 6), soma das categorias (Alg. 8) |
| Vetor | Dígitos do CPF (Alg. 2) |
| Matriz | Mapa de assentos fileira × número (Alg. 3, 4 e 5b) |
| Constante | `DESCONTO_MEIA` (Alg. 4) |
| Contadores e acumuladores | Relatório do evento (Alg. 6), soma das quantidades (Alg. 8) |
| Transação (tudo ou nada) | Compra (Alg. 4), cancelamento (Alg. 5), remarcação (Alg. 5b), autocadastro (Alg. 9) |
