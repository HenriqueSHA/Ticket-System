# Roteiro da apresentação — 30/11/2026 (Equipe 11)

Tempo sugerido: **12 minutos** de apresentação + perguntas. Divisão sugerida (a equipe pode trocar).

| Tempo | Quem | Slide | O que falar |
|---|---|---|---|
| 0:00–1:30 | Yasmin | 1–2 | Equipe, tema e problema: venda duplicada, assentos sem controle, dados espalhados; três perfis (cliente, organizador, SuperAdmin) |
| 1:30–3:00 | Yasmin | 3 | Objetivos e requisitos (seções 4 e 5); decomposição em módulos (seção 6) |
| 3:00–5:00 | Fellipe | 4–5 | Algoritmo da compra (pseudocódigo + fluxograma 4): decisões em sequência e transação; matriz de assentos com dois laços `PARA` |
| 5:00–6:30 | Fellipe | 6 | Estruturas de dados (vetor do CPF, matriz, listas, dicionário) e POO (classes, encapsulamento, polimorfismo com a Strategy da meia-entrada) |
| 6:30–9:30 | Henrique | 7 | **Demonstração ao vivo** (roteiro abaixo) |
| 9:30–11:00 | Henrique | 8–9 | Arquitetura (3 contêineres), banco e transação contra venda duplicada; testes (55, CI no GitHub) |
| 11:00–12:00 | Yasmin | 10 | Limitações, melhorias e conclusão |

## Demonstração ao vivo (3 minutos)

Antes: `docker compose up -d` e abrir http://localhost:3000. Se a internet ou o projetor falharem,
use o vídeo `documentos/relatorio/demonstracao.mp4`.

1. **Organizador novo:** "Sou organizador" → CNPJ `12.345.678/0001-00` mostra o erro do dígito
   verificador → corrigir para `12.ABC.345/01DE-35` (CNPJ alfanumérico) → conta criada.
2. Entrar e cadastrar um evento com uma categoria de assento marcado (4 × 10). Mostrar a tela de
   gestão: capacidade, vendidos e disponíveis.
3. **Cliente** `ana@email.com` / `ana12345`: Show de Rock → Cadeira → meia-entrada → assento B2 no mapa
   → resumo (R$ 90,00) → confirmar → comprovante.
4. "Meus ingressos" → **Trocar assento** → C3 → o B2 volta a ficar livre.
5. **Organizador** `contato@soleventos.com` / `sol12345`: relatório do Show (vendidos, ocupação,
   receita, mapa).
6. **SuperAdmin** `admin@ingressos.com` / `admin123`: lista de organizadores; bloquear e ativar.

## Perguntas prováveis

- **Por que Rust?** A linguagem era livre. Rust evita valor nulo e erros de memória já na compilação, e
  o compilador obriga a tratar todos os erros, o que combina com as regras de validação do trabalho.
- **Onde está a POO se Rust não tem classe?** `struct` + `impl` = classe; atributos privados =
  encapsulamento; `trait` = interface/polimorfismo; composição no lugar de herança (seção 10).
- **Como garantem que não há venda duplicada?** Três camadas: verificação no algoritmo, transação
  com `UPDATE ... WHERE situacao = 'LIVRE'` e índice único no banco (seção 13.4, teste 17).
- **Por que dinheiro em centavos?** Para não ter erro de arredondamento do ponto flutuante (seção 9.4).
- **O que falta?** Pagamento real, QR Code, e-mail e hospedagem na nuvem (seção 17).
