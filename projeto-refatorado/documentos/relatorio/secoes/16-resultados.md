# 16. Resultados e análise

## 16.1 Evidências de funcionamento

As telas abaixo foram capturadas da aplicação em execução (três contêineres, banco PostgreSQL) por um
roteiro automático que age como um usuário (`ferramentas/demonstracao.mjs`). O mesmo roteiro grava
um vídeo curto da demonstração.

![Página inicial: entrada no sistema e criação de conta de cliente ou de organizador](../prints/01-inicio.png)

![Autocadastro de organizador recusando um CNPJ com dígitos verificadores errados (Alg. 9)](../prints/02-erro-cnpj-invalido.png)

![Conta de organizador criada com o CNPJ alfanumérico 12.ABC.345/01DE-35](../prints/03-conta-de-organizador-criada.png)

![Cadastro de evento com categoria de assento marcado (Alg. 3)](../prints/04-cadastro-de-evento.png)

![Gestão do evento: capacidade, vendidos, disponíveis e categorias (Alg. 8)](../prints/05-gerenciar-evento.png)

![Lista de eventos disponíveis para o cliente](../prints/06-eventos-disponiveis.png)

![Compra: categoria, meia-entrada e escolha do assento B2 no mapa (matriz) — Alg. 4](../prints/07-compra-mapa-de-assentos.png)

![Resumo antes de confirmar: meia-entrada de R$ 180,00 = R$ 90,00](../prints/08-resumo-da-compra.png)

![Comprovante da compra](../prints/09-comprovante.png)

![Histórico de compras do cliente, com as opções de trocar assento e cancelar](../prints/10-meus-ingressos.png)

![Remarcação: escolha do novo assento C3 (Alg. 5b)](../prints/11-remarcar-assento.png)

![Assento trocado de B2 para C3, com o mesmo valor](../prints/12-assento-trocado.png)

![Relatório do evento para o organizador: vendidos, ocupação, receita e mapa (Alg. 6)](../prints/13-relatorio-do-evento.png)

![Painel do SuperAdmin com os organizadores e as ações de bloquear, editar e remover (Alg. 7)](../prints/14-superadmin-organizadores.png)

![Erro tratado: e-mail ou senha inválidos (Alg. 1)](../prints/15-erro-login-invalido.png)

## 16.2 Análise

| Problema apresentado (seção 2) | Como a solução trata | Evidência |
|---|---|---|
| Venda duplicada de ingressos | Verificação do assento + transação + índice único no banco | Testes 5 e 17; o assento vendido aparece com ✕ e não pode ser escolhido |
| Controle de disponibilidade de assentos | Matriz de assentos e quantidade disponível por categoria | Figuras de compra, remarcação e relatório |
| Processos de compra lentos e pouco transparentes | Compra em 4 passos com resumo antes de confirmar e comprovante | Figuras do resumo e do comprovante |
| Desorganização dos dados | Banco relacional com restrições e um único sistema para os três perfis | Seção 13; painel do SuperAdmin |
| Organizador precisa acompanhar as vendas | Relatório com vendidos, cancelados, presença, receita e ocupação | Figura do relatório; teste 16 |

Os resultados obtidos nos 19 cenários da seção 15 foram iguais aos esperados, e os 55 testes
automáticos passam no pipeline de CI. Todos os objetivos específicos da seção 4.2 foram atendidos,
incluindo a remarcação prevista nas regras 7.3 e 7.4, que foi implementada nesta etapa.
