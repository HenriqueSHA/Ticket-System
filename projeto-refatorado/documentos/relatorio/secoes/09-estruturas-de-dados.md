# 9. Estruturas de dados utilizadas

## 9.1 Visão geral

A escolha de cada estrutura partiu da pergunta "como esse dado é usado pelo algoritmo?". Dados
consultados por posição viraram vetores ou matrizes, dados com quantidade variável viraram listas
dinâmicas e dados que precisam ser achados por uma chave viraram tabelas de dispersão (dicionários).

| Estrutura | Tipo em Rust | Onde é usada | Por que foi escolhida |
|---|---|---|---|
| Variáveis e constantes | `let`, `const` | `DESCONTO_MEIA = 50`, contadores do relatório, limites `MAX_FILEIRAS` | Valores simples e regras fixas ficam com nome próprio e não se espalham pelo código |
| Vetor | `Vec<u32>` (11 ou 14 posições) | Dígitos do CPF e do CNPJ (Alg. 2) | Cada dígito é acessado pela posição para calcular os dígitos verificadores |
| Matriz | `Vec<Vec<Situacao>>` | Mapa de assentos `MapaAssentos` (Alg. 3, 4 e 5b) | O salão é naturalmente uma grade fileira × número; dois laços aninhados percorrem e desenham o mapa |
| Lista dinâmica | `Vec<T>` | Categorias de um evento, vendas de um evento, histórico do cliente | A quantidade de itens só é conhecida durante a execução |
| Registro (struct) | `struct Evento`, `struct Venda`... | Todas as entidades da seção 7.2 | Agrupa os atributos de cada entidade com tipos definidos |
| Enumeração | `enum Perfil`, `StatusVenda`, `Situacao`... | Perfis, status e situações | Restringe o valor a uma lista fechada; o compilador obriga a tratar todos os casos no `match` |
| Opcional | `Option<T>` | Assento da venda (Pista não tem), registro buscado que pode não existir | Substitui o `NULO` do pseudocódigo sem risco de acesso a valor inexistente |
| Dicionário (tabela hash) | `HashMap<String, (Sessao, Instant)>` | Sessões abertas e avisos no front | Acha a sessão pelo token em tempo constante; guarda o instante para expirar |
| Tabelas relacionais | PostgreSQL | Persistência (seção 13) | Mantém os dados entre execuções e garante as regras com restrições e transações |

## 9.2 Matriz de assentos

A categoria com assento marcado é descrita por um setor com `fileiras × assentos_por_fileira`. O
mapa é montado como uma matriz em que a linha é a fileira (A, B, C...) e a coluna é o número:

```
matriz ← nova matriz [fileiras][assentosPorFileira] preenchida com LIVRE
PARA CADA assento EM assentosDaCategoria FAÇA
    f ← indiceDaFileira(assento.fileira)        // "A" → 0, "B" → 1...
    n ← assento.numero - 1
    matriz[f][n] ← assento.situacao
FIMPARA
```

A conversão de letra para índice usa a tabela ASCII (`'B' - 'A' = 1`). Como o tamanho máximo é
26 fileiras × 30 assentos, a matriz tem no máximo 780 posições.

## 9.3 Vetor de dígitos do CPF

O CPF é transformado em um vetor de 11 inteiros. O primeiro dígito verificador usa os 9 primeiros
elementos com pesos 10 a 2, e o segundo usa os 10 primeiros com pesos 11 a 2. O CNPJ usa a mesma
ideia com 14 posições, e no formato alfanumérico cada caractere vale o seu código ASCII − 48.

## 9.4 Valores monetários

Os preços e valores são guardados como **inteiros em centavos** (`Dinheiro(i64)`), e não como número
real. Isso evita os erros de arredondamento do ponto flutuante (em binário, 0,1 + 0,2 não é
exatamente 0,3). O texto digitado ("1.234,50") é convertido só com aritmética inteira:
`reais × 100 + centavos`.
