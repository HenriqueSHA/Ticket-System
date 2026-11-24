# 17. Limitações

- **Pagamento simulado:** a forma de pagamento (PIX, crédito ou débito) é apenas registrada; não há
  integração com uma operadora de pagamento nem estorno real no cancelamento.
- **Meia-entrada sem comprovação:** o sistema aplica o desconto de 50%, mas não confere documento de
  estudante ou de outro direito à meia-entrada.
- **Ingresso sem QR Code:** o check-in é feito pelo número da compra; não há leitura de código.
- **Sem recuperação de senha nem confirmação de e-mail:** o sistema não envia e-mails.
- **Remarcação limitada:** a troca de assento acontece dentro da mesma categoria; mudar para outra
  categoria (com diferença de preço) exige cancelar e comprar de novo.
- **Sessões em memória:** o front guarda as sessões abertas em memória; ao reiniciar, ele as recupera
  pelo back, mas com vários servidores de front seria preciso um armazenamento compartilhado.
- **Mapa de assentos retangular:** cada categoria é uma grade de até 26 fileiras × 30 assentos; salas
  com formato irregular não são representadas.
- **Hospedagem:** o pipeline já publica as imagens e tem a etapa de implantação por SSH pronta, mas o
  servidor na nuvem (AWS) ainda não foi criado; a aplicação roda localmente com Docker.
