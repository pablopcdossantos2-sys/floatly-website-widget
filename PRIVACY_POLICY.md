# Política de Privacidade — Floatly

**Última atualização:** 30 de outubro de 2026


## 1. Sobre o Floatly

Floatly é um aplicativo gratuito para Windows que exibe, em uma janela compacta, um site escolhido pelo próprio usuário. O aplicativo não exige a criação de uma conta Floatly.

O Floatly não cobra pelo uso, não oferece compras dentro do aplicativo nem assinaturas e não inclui um sistema próprio de publicidade. O conteúdo de anúncios que eventualmente apareça dentro de um site escolhido pelo usuário é controlado por esse site, não pelo Floatly.

Esta política descreve o tratamento de informações realizado pelo próprio aplicativo e por seu publicador. Ela não substitui as políticas de privacidade dos sites, provedores de autenticação, serviços do Windows, Microsoft Edge WebView2 ou outros serviços de terceiros que o usuário decida utilizar.

## 2. Responsável pelo tratamento e contato

**Responsável pelo Floatly (controlador):** Pablo Phillipe Cândido dos Santos

**Contato para privacidade e proteção de dados:** pablopcsantos@outlook.com

O responsável acima é o canal para dúvidas e solicitações relacionadas a esta política e ao tratamento de dados realizado pelo Floatly. O nome informado deve corresponder à pessoa ou entidade que efetivamente publica e mantém o aplicativo.

## 3. Quais informações o Floatly utiliza

O Floatly foi desenvolvido para funcionar localmente no computador. O código do aplicativo não implementa uma conta de usuário, um serviço de sincronização em nuvem, uma plataforma própria de análise de uso, publicidade comportamental ou envio de telemetria e registros de diagnóstico para servidores do publicador.

Para fornecer seus recursos, o aplicativo pode armazenar ou processar as seguintes informações no dispositivo:

### 3.1. Configurações do aplicativo

O Floatly salva localmente as informações necessárias para manter a configuração escolhida pelo usuário, tais como:

- endereço HTTP ou HTTPS do site principal escolhido pelo usuário;
- largura, altura e posição da janela;
- idioma da interface;
- nível de zoom;
- preferência de inicialização com o Windows;
- opções de janela, como “sempre no topo”, exibição na barra de tarefas e bloqueio de movimentação;
- intervalo de recarga automática;
- atalho global e preferências visuais.

Essas informações são usadas para abrir o site escolhido e restaurar as preferências nas próximas execuções. Elas são mantidas em um arquivo de configuração no perfil local do usuário do Windows, normalmente em %APPDATA%\WidgetWebsite\config.json.

### 3.2. Registros técnicos e de diagnóstico

Para identificar falhas e investigar congelamentos, o aplicativo cria registros técnicos no computador. Eles podem conter:

- data e horário dos eventos;
- inicialização e encerramento do aplicativo;
- estado e atividade do WebView2;
- mensagens de erro e informações técnicas do Windows;
- eventos de navegação, incluindo esquema, domínio, porta e parte do caminho da URL.

Ao registrar uma URL, o Floatly remove credenciais, parâmetros de consulta e fragmentos e limita o trecho do caminho registrado. **Isso reduz, mas não elimina, a possibilidade de uma URL revelar informação pessoal:** um caminho pode conter um nome, identificador ou outro dado inserido pelo próprio site.

Algumas mensagens técnicas também podem conter caminhos de arquivos do perfil do Windows, que eventualmente incluem o nome da conta local. Os registros ficam no dispositivo, em diagnostics.log, dentro da pasta de dados do Floatly. O aplicativo faz rotação dos arquivos para limitar seu crescimento e manter um número limitado de cópias anteriores.

O objetivo desses registros é diagnóstico, estabilidade e suporte técnico. O aplicativo não os envia automaticamente ao publicador. Caso o usuário decida compartilhar um registro voluntariamente para pedir suporte, deve revisá-lo antes e remover qualquer informação que não queira divulgar.

### 3.3. Sessão, cookies e dados do site escolhido

O site aberto no Floatly é executado no Microsoft Edge WebView2. Como em um navegador, o próprio site pode criar cookies, armazenamento local, cache, identificadores de sessão e outros dados necessários às suas páginas e aos seus processos de autenticação.

O Floatly mantém os dados do perfil WebView2 localmente para que a sessão do site possa continuar entre execuções. A janela auxiliar de autenticação reconhecida pelo aplicativo utiliza a mesma pasta de dados WebView2, permitindo que o fluxo de login compartilhe a sessão com a janela principal.

Esses dados são controlados, em grande parte, pelo site e pelos provedores de autenticação utilizados. O código próprio do Floatly não lê, copia nem envia senhas ao publicador. Entretanto, o site pode manter cookies ou outros dados de sessão no perfil local do WebView2, conforme o funcionamento do próprio serviço.

A pasta de dados do WebView2 fica normalmente em %APPDATA%\WidgetWebsite\webview-data.

### 3.4. Verificação de conexão

Enquanto o site estiver aberto, o Floatly realiza verificações técnicas periódicas de disponibilidade, incluindo uma solicitação ao endereço /favicon.ico na origem do site configurado. Essa solicitação é feita diretamente ao servidor do site escolhido, para verificar se ele continua acessível.

Como qualquer acesso a um site, essa comunicação pode revelar ao servidor de destino o endereço IP do dispositivo, o horário da solicitação e informações técnicas da conexão. O publicador do Floatly não recebe esses dados por meio de um servidor próprio.

## 4. Como as informações são utilizadas

As informações locais descritas acima são utilizadas somente para:

- carregar o endereço escolhido pelo usuário;
- preservar configurações e preferências entre execuções;
- manter a sessão local do site no WebView2;
- verificar a disponibilidade do site aberto;
- detectar e diagnosticar falhas, congelamentos e problemas de funcionamento.

O Floatly não utiliza essas informações para criar perfis de publicidade, vender dados, rastrear a navegação do usuário em outros aplicativos ou alimentar um serviço de análise de uso operado pelo publicador.

## 5. Com quem as informações podem ser compartilhadas

O Floatly não possui um servidor próprio para receber as configurações, cookies, conteúdo de navegação ou registros de diagnóstico do usuário. O aplicativo não vende nem disponibiliza essas informações ao publicador para fins comerciais.

Ao utilizar o Floatly, algumas informações podem ser transmitidas diretamente a terceiros por causa da navegação escolhida pelo usuário:

- **Site escolhido:** recebe as solicitações necessárias para carregar suas páginas e recursos. O site pode tratar endereço IP, dados do navegador, cookies, informações de sessão e outras informações conforme sua própria política.
- **Provedores de autenticação:** quando o usuário inicia um login, o provedor correspondente pode receber as informações necessárias para autenticar a sessão. A janela auxiliar de autenticação compartilha o perfil local do WebView2.
- **Sites de destino de links externos:** links para outros domínios que não fazem parte do fluxo de autenticação tratado pelo Floatly são encaminhados ao navegador padrão do Windows. A partir daí, a navegação segue as regras e políticas desse navegador e dos sites acessados.
- **Microsoft, Windows e WebView2:** o funcionamento do aplicativo depende do Windows e do Microsoft Edge WebView2 Runtime. A Microsoft pode tratar informações técnicas relacionadas ao sistema, à instalação, à atualização ou ao funcionamento desses componentes de acordo com seus próprios termos e avisos de privacidade.

As páginas abertas podem incluir conteúdo, scripts ou recursos fornecidos por outros domínios. Esses terceiros podem receber dados de conexão quando o navegador carrega seus recursos. O Floatly não controla as práticas de privacidade desses serviços.

## 6. Armazenamento, segurança e retenção

As configurações, os registros de diagnóstico e os dados de sessão descritos nesta política são mantidos localmente no perfil do usuário do Windows. O Floatly não mantém uma cópia desses dados em servidores do publicador.

O aplicativo utiliza os mecanismos normais de armazenamento e permissões do Windows e do WebView2. O Floatly não adiciona uma camada própria de criptografia aos arquivos locais de configuração e diagnóstico. Por isso, a segurança desses arquivos também depende da proteção da conta do Windows, do dispositivo e do acesso físico ao computador.

A conexão com o site escolhido depende do endereço informado pelo usuário. Prefira endereços HTTPS. Uma página acessada por HTTP não oferece a mesma proteção de criptografia em trânsito que uma conexão HTTPS.

Os dados locais permanecem no dispositivo até que o usuário os remova ou desinstale o aplicativo com a opção de exclusão dos dados selecionada. Os arquivos de diagnóstico são rotacionados automaticamente, com limite de tamanho e de cópias anteriores; não há prazo de retenção em servidor do publicador porque esses registros não são enviados a ele.

## 7. Controles disponíveis ao usuário

O usuário pode controlar os dados e a utilização do Floatly das seguintes maneiras:

- **Alterar o site:** substituir ou remover o endereço configurado nas configurações do aplicativo.
- **Encerrar a sessão do site:** fechar o Floatly não garante, por si só, a exclusão de cookies ou dados de sessão. Esses dados permanecem no perfil local até serem removidos.
- **Remover somente os dados de navegação do Floatly:** com o aplicativo totalmente encerrado, excluir a pasta %APPDATA%\WidgetWebsite\webview-data. Isso remove a sessão local do WebView2 e pode exigir novo login nos sites. O usuário deve fazer uma cópia antes, caso deseje preservar esses dados.
- **Remover todos os dados do aplicativo:** ao desinstalar, selecionar a opção do instalador para excluir os dados do aplicativo. Essa opção remove as pastas locais de dados do Floatly, incluindo configurações, registros e sessão WebView2.
- **Remover manualmente os dados:** com o aplicativo encerrado, excluir as pastas %APPDATA%\WidgetWebsite e, se existir, %LOCALAPPDATA%\WidgetWebsite. A exclusão remove configurações, diagnósticos e dados de sessão.
- **Controlar o que é compartilhado com sites:** escolher cuidadosamente o endereço utilizado, revisar as políticas dos sites e provedores de login e abrir links externos no navegador padrão.

A exclusão de dados locais pode não apagar informações que já tenham sido recebidas e armazenadas pelos sites ou provedores de autenticação. Para essas informações, o usuário deve entrar em contato diretamente com o respectivo serviço.

## 8. Direitos de proteção de dados

Quando a legislação aplicável, incluindo a Lei Geral de Proteção de Dados Pessoais (LGPD) do Brasil, conceder direitos ao titular, ele poderá solicitar ao responsável pelo Floatly, conforme aplicável:

- confirmação da existência de tratamento e acesso às informações;
- correção de dados pessoais incompletos, inexatos ou desatualizados;
- informação sobre o uso e o compartilhamento de dados;
- anonimização, bloqueio ou eliminação de dados tratados em desconformidade com a legislação, quando cabível;
- eliminação dos dados tratados com consentimento, ressalvadas as hipóteses legais de conservação;
- revisão de decisões e demais direitos previstos na legislação aplicável.

As solicitações relacionadas ao tratamento realizado pelo publicador devem ser encaminhadas ao contato indicado na seção 2. Como a maior parte dos dados do Floatly permanece somente no computador do usuário, o publicador não consegue consultar ou apagar remotamente arquivos locais. O usuário pode utilizar os controles descritos na seção 7.

Dados tratados diretamente por sites, provedores de autenticação, Microsoft ou outros terceiros devem ser solicitados aos respectivos responsáveis.

## 9. Crianças e adolescentes

O Floatly é uma ferramenta geral de navegação compacta para Windows, não um serviço de contas ou uma plataforma destinada especificamente a crianças. O aplicativo não solicita intencionalmente informações pessoais de crianças. Sites acessados pelo usuário podem, contudo, apresentar conteúdo ou solicitar informações próprias; essas atividades são regidas pelas políticas e controles dos respectivos serviços e pela legislação aplicável.

## 10. Alterações nesta política

Esta política poderá ser atualizada quando houver mudanças nas funcionalidades do Floatly, nas práticas de tratamento de dados ou nas exigências legais e da Microsoft Store. A data de atualização será alterada no início do documento. Mudanças relevantes deverão ser comunicadas por meio dos canais públicos do projeto e/ou da página do aplicativo, conforme apropriado.

## 11. Contato

Para dúvidas, solicitações de privacidade ou exercício de direitos relacionados ao tratamento de dados realizado pelo Floatly, entre em contato:

**Responsável:** Pablo Phillipe Cândido dos Santos  
**E-mail:** pablopcsantos@outlook.com

---

# Privacy Policy — Floatly

**Last updated:** October 30, 2026


## 1. About Floatly

Floatly is a free Windows application that displays a website selected by the user in a compact window. The application does not require a Floatly account.

Floatly is free to use, has no in-app purchases or subscriptions, and does not include its own advertising system. Any advertisements that may appear within a website selected by the user are controlled by that website, not by Floatly.

This policy describes information handling by the Floatly application and its publisher. It does not replace the privacy policies of websites, authentication providers, Windows services, Microsoft Edge WebView2, or other third-party services the user chooses to access.

## 2. Controller and contact

**Floatly controller:** Pablo Phillipe Cândido dos Santos

**Privacy and data protection contact:** pablopcsantos@outlook.com

The controller identified above is the contact for questions and requests about this policy and the processing carried out by Floatly. The name must match the person or entity that actually publishes and maintains the application.

## 3. Information Floatly uses

Floatly is designed to operate locally on the computer. The application's own code does not implement a Floatly user account, cloud synchronization service, publisher-operated usage analytics platform, behavioral advertising, or the transmission of telemetry and diagnostic logs to the publisher's servers.

To provide its features, the application may store or process the following information on the device:

### 3.1. Application settings

Floatly locally saves information needed to preserve the user's choices, such as:

- the HTTP or HTTPS address of the user's primary website;
- window width, height, and position;
- interface language;
- zoom level;
- Windows startup preference;
- window options such as always-on-top, taskbar visibility, and movement lock;
- automatic reload interval;
- global shortcut and visual preferences.

This information is used to open the selected website and restore preferences on subsequent launches. It is stored in a configuration file in the user's local Windows profile, normally at %APPDATA%\WidgetWebsite\config.json.

### 3.2. Technical and diagnostic logs

To identify failures and investigate freezes, the application creates technical logs on the computer. They may contain:

- event dates and times;
- application startup and shutdown;
- WebView2 activity and status;
- error messages and technical Windows information;
- navigation events, including URL scheme, domain, port, and part of the URL path.

When recording a URL, Floatly removes credentials, query parameters, and fragments, and limits the recorded portion of the path. **This reduces, but does not eliminate, the possibility that a URL reveals personal information:** a path may contain a name, identifier, or other information supplied by the website.

Some technical messages may also contain local file paths, which can sometimes include the name of the Windows account. Logs are stored on the device in diagnostics.log, inside Floatly's data folder. The application rotates log files to limit their size and retain only a limited number of previous copies.

These logs are used for diagnostics, stability, and technical support. The application does not automatically send them to the publisher. If a user voluntarily shares a log when requesting support, they should review it first and remove any information they do not wish to disclose.

### 3.3. Session, cookies, and data from the selected website

The website opened in Floatly runs in Microsoft Edge WebView2. As in a browser, the website itself may create cookies, local storage, cache, session identifiers, and other data needed by its pages and authentication processes.

Floatly keeps the WebView2 profile data locally so the website session can continue between launches. The auxiliary authentication window recognized by the application uses the same WebView2 data folder, allowing a sign-in flow to share its session with the main window.

These data are largely controlled by the website and its authentication providers. Floatly's own code does not read, copy, or send passwords to the publisher. However, the website may store cookies or other session data in the local WebView2 profile, depending on how that service works.

The WebView2 data folder is normally located at %APPDATA%\WidgetWebsite\webview-data.

### 3.4. Connection checks

While the website is open, Floatly performs periodic technical availability checks, including a request to /favicon.ico at the origin of the configured website. This request is sent directly to the selected website's server to check whether it is still reachable.

Like any website access, this communication may reveal the device's IP address, the request time, and technical connection information to the destination server. The Floatly publisher does not receive this information through its own server.

## 4. How information is used

The local information described above is used only to:

- load the address selected by the user;
- preserve settings and preferences between launches;
- maintain the website's local WebView2 session;
- check the availability of the open website;
- detect and diagnose failures, freezes, and operational problems.

Floatly does not use this information to create advertising profiles, sell data, track the user's browsing in other applications, or feed a usage analytics service operated by the publisher.

## 5. Who information may be shared with

Floatly does not have its own server for receiving users' settings, cookies, browsing content, or diagnostic logs. The application does not sell or make this information available to the publisher for commercial purposes.

When using Floatly, some information may be transmitted directly to third parties as a result of the website navigation chosen by the user:

- **Selected website:** receives the requests needed to load its pages and resources. It may process the IP address, browser data, cookies, session information, and other information under its own policy.
- **Authentication providers:** when the user signs in, the relevant provider may receive information needed to authenticate the session. The auxiliary authentication window shares the local WebView2 profile.
- **External link destinations:** links to other domains that are not part of an authentication flow handled by Floatly are sent to the Windows default browser. From that point on, browsing follows that browser's and the destination websites' policies.
- **Microsoft, Windows, and WebView2:** the application depends on Windows and Microsoft Edge WebView2 Runtime. Microsoft may process technical information related to the system, installation, updating, or operation of those components under its own terms and privacy notices.

Pages opened by the user may include content, scripts, or resources supplied by other domains. Those third parties may receive connection data when the browser loads their resources. Floatly does not control their privacy practices.

## 6. Storage, security, and retention

The settings, diagnostic logs, and session data described in this policy are stored locally in the user's Windows profile. Floatly does not keep a copy of these data on the publisher's servers.

The application uses the normal storage and permission mechanisms of Windows and WebView2. Floatly does not add its own encryption layer to local configuration and diagnostic files. Therefore, the security of these files also depends on protection of the Windows account, device, and physical access to the computer.

Connection security depends on the address entered by the user. HTTPS addresses are recommended. A website accessed over HTTP does not provide the same in-transit encryption as an HTTPS connection.

Local data remain on the device until the user removes them or uninstalls the application with the data deletion option selected. Diagnostic files are automatically rotated, with limits on file size and previous copies. There is no publisher-server retention period because these logs are not sent to the publisher.

## 7. User controls

Users can control Floatly data and use in the following ways:

- **Change the website:** replace or remove the configured address in the application settings.
- **End a website session:** closing Floatly does not by itself guarantee deletion of cookies or session data. These remain in the local profile until removed.
- **Remove only Floatly browsing data:** with the application fully closed, delete the %APPDATA%\WidgetWebsite\webview-data folder. This removes the local WebView2 session and may require signing in to websites again. Make a backup first if you wish to preserve this data.
- **Remove all application data:** when uninstalling, select the installer option to delete application data. This removes Floatly's local data folders, including settings, logs, and the WebView2 session.
- **Manually remove data:** with the application closed, delete %APPDATA%\WidgetWebsite and, if it exists, %LOCALAPPDATA%\WidgetWebsite. This removes settings, diagnostics, and session data.
- **Control what is shared with websites:** carefully choose the website address, review the policies of websites and sign-in providers, and open external links in the default browser.

Deleting local data may not remove information already received and stored by websites or authentication providers. Users should contact those services directly about such information.

## 8. Data protection rights

Where applicable law, including Brazil's General Data Protection Law (LGPD), grants data subject rights, users may request from the Floatly controller, as applicable:

- confirmation of processing and access to information;
- correction of incomplete, inaccurate, or outdated personal data;
- information about data use and sharing;
- anonymization, blocking, or deletion of data processed in violation of applicable law, where appropriate;
- deletion of data processed on the basis of consent, subject to legal retention exceptions;
- review of decisions and other rights provided by applicable law.

Requests about processing carried out by the publisher should be sent to the contact in Section 2. Since most Floatly data remains only on the user's computer, the publisher cannot remotely access or delete local files. Users can use the controls described in Section 7.

Data processed directly by websites, authentication providers, Microsoft, or other third parties must be requested from those respective parties.

## 9. Children and teenagers

Floatly is a general-purpose compact browsing tool for Windows, not an account service or a platform specifically designed for children. The application does not intentionally request personal information from children. However, websites accessed by the user may display content or request information of their own; those activities are governed by the policies and controls of the respective services and applicable law.

## 10. Changes to this policy

This policy may be updated when Floatly features, data handling practices, or legal and Microsoft Store requirements change. The update date will be changed at the beginning of the document. Material changes should be communicated through the project's public channels and/or the application's page, as appropriate.

## 11. Contact

For questions, privacy requests, or to exercise rights related to data processing carried out by Floatly, contact:

**Controller:** Pablo Phillipe Cândido dos Santos  
**Email:** pablopcsantos@outlook.com

---

# Política de privacidad — Floatly

**Última actualización:** 30 de octubre de 2026


## 1. Acerca de Floatly

Floatly es una aplicación gratuita para Windows que muestra un sitio web elegido por el propio usuario en una ventana compacta. La aplicación no requiere crear una cuenta de Floatly.

Floatly es gratuita, no ofrece compras dentro de la aplicación ni suscripciones y no incluye un sistema propio de publicidad. Los anuncios que puedan aparecer dentro de un sitio web elegido por el usuario están controlados por ese sitio, no por Floatly.

Esta política describe el tratamiento de información realizado por la aplicación Floatly y su editor. No sustituye las políticas de privacidad de los sitios web, proveedores de autenticación, servicios de Windows, Microsoft Edge WebView2 u otros servicios de terceros que el usuario decida utilizar.

## 2. Responsable y contacto

**Responsable de Floatly (responsable del tratamiento):** Pablo Phillipe Cândido dos Santos

**Contacto para privacidad y protección de datos:** pablopcsantos@outlook.com

El responsable indicado arriba es el canal para preguntas y solicitudes sobre esta política y el tratamiento realizado por Floatly. El nombre debe corresponder a la persona o entidad que realmente publica y mantiene la aplicación.

## 3. Qué información utiliza Floatly

Floatly está diseñada para funcionar localmente en el ordenador. El código propio de la aplicación no implementa una cuenta de usuario de Floatly, un servicio de sincronización en la nube, una plataforma de análisis de uso operada por el editor, publicidad comportamental ni el envío de telemetría y registros de diagnóstico a servidores del editor.

Para ofrecer sus funciones, la aplicación puede almacenar o procesar la siguiente información en el dispositivo:

### 3.1. Configuración de la aplicación

Floatly guarda localmente la información necesaria para conservar las elecciones del usuario, por ejemplo:

- dirección HTTP o HTTPS del sitio web principal elegido;
- ancho, alto y posición de la ventana;
- idioma de la interfaz;
- nivel de zoom;
- preferencia de inicio con Windows;
- opciones de la ventana, como permanecer siempre encima, mostrarse en la barra de tareas y bloquear el movimiento;
- intervalo de recarga automática;
- atajo global y preferencias visuales.

Esta información se utiliza para abrir el sitio elegido y restaurar las preferencias en los siguientes inicios. Se almacena en un archivo de configuración dentro del perfil local del usuario de Windows, normalmente en %APPDATA%\WidgetWebsite\config.json.

### 3.2. Registros técnicos y de diagnóstico

Para identificar fallos e investigar bloqueos, la aplicación crea registros técnicos en el ordenador. Pueden contener:

- fechas y horas de los eventos;
- inicio y cierre de la aplicación;
- actividad y estado de WebView2;
- mensajes de error e información técnica de Windows;
- eventos de navegación, incluido el esquema, dominio, puerto y parte de la ruta de la URL.

Al registrar una URL, Floatly elimina las credenciales, los parámetros de consulta y los fragmentos, y limita la parte de la ruta registrada. **Esto reduce, pero no elimina, la posibilidad de que una URL revele información personal:** una ruta puede contener un nombre, identificador u otra información introducida por el propio sitio.

Algunos mensajes técnicos también pueden contener rutas de archivos locales, que en ocasiones incluyen el nombre de la cuenta de Windows. Los registros se guardan en el dispositivo en diagnostics.log, dentro de la carpeta de datos de Floatly. La aplicación rota los archivos para limitar su tamaño y conservar solo un número limitado de copias anteriores.

Estos registros se utilizan para diagnóstico, estabilidad y soporte técnico. La aplicación no los envía automáticamente al editor. Si el usuario decide compartir voluntariamente un registro al solicitar soporte, debe revisarlo primero y eliminar cualquier información que no quiera divulgar.

### 3.3. Sesión, cookies y datos del sitio elegido

El sitio web abierto en Floatly se ejecuta en Microsoft Edge WebView2. Como en un navegador, el propio sitio puede crear cookies, almacenamiento local, caché, identificadores de sesión y otros datos necesarios para sus páginas y procesos de autenticación.

Floatly conserva localmente los datos del perfil WebView2 para que la sesión del sitio pueda continuar entre ejecuciones. La ventana auxiliar de autenticación reconocida por la aplicación utiliza la misma carpeta de datos de WebView2, lo que permite compartir la sesión con la ventana principal.

Estos datos están controlados en gran medida por el sitio web y sus proveedores de autenticación. El código propio de Floatly no lee, copia ni envía contraseñas al editor. Sin embargo, el sitio puede almacenar cookies u otros datos de sesión en el perfil local de WebView2, según el funcionamiento de ese servicio.

La carpeta de datos de WebView2 suele estar en %APPDATA%\WidgetWebsite\webview-data.

### 3.4. Comprobaciones de conexión

Mientras el sitio está abierto, Floatly realiza comprobaciones técnicas periódicas de disponibilidad, incluida una solicitud a /favicon.ico en el origen del sitio configurado. Esta solicitud se envía directamente al servidor del sitio elegido para comprobar si sigue accesible.

Como cualquier acceso a un sitio web, esta comunicación puede revelar al servidor de destino la dirección IP del dispositivo, la hora de la solicitud y datos técnicos de la conexión. El editor de Floatly no recibe esta información mediante un servidor propio.

## 4. Cómo se utiliza la información

La información local descrita anteriormente se utiliza únicamente para:

- cargar la dirección elegida por el usuario;
- conservar la configuración y las preferencias entre ejecuciones;
- mantener la sesión local del sitio en WebView2;
- comprobar la disponibilidad del sitio abierto;
- detectar y diagnosticar fallos, bloqueos y problemas de funcionamiento.

Floatly no utiliza esta información para crear perfiles publicitarios, vender datos, rastrear la navegación del usuario en otras aplicaciones ni alimentar un servicio de análisis de uso operado por el editor.

## 5. Con quién puede compartirse la información

Floatly no dispone de un servidor propio para recibir la configuración, las cookies, el contenido de navegación o los registros de diagnóstico de los usuarios. La aplicación no vende ni pone esta información a disposición del editor con fines comerciales.

Al utilizar Floatly, parte de la información puede transmitirse directamente a terceros debido a la navegación elegida por el usuario:

- **Sitio web elegido:** recibe las solicitudes necesarias para cargar sus páginas y recursos. Puede tratar la dirección IP, datos del navegador, cookies, información de sesión y otros datos conforme a su propia política.
- **Proveedores de autenticación:** cuando el usuario inicia sesión, el proveedor correspondiente puede recibir la información necesaria para autenticar la sesión. La ventana auxiliar de autenticación comparte el perfil local de WebView2.
- **Destinos de enlaces externos:** los enlaces a otros dominios que no forman parte de un flujo de autenticación gestionado por Floatly se envían al navegador predeterminado de Windows. A partir de ese momento, la navegación sigue las políticas de ese navegador y de los sitios de destino.
- **Microsoft, Windows y WebView2:** la aplicación depende de Windows y Microsoft Edge WebView2 Runtime. Microsoft puede tratar información técnica relacionada con el sistema, la instalación, la actualización o el funcionamiento de esos componentes conforme a sus propios términos y avisos de privacidad.

Las páginas abiertas pueden incluir contenido, scripts o recursos proporcionados por otros dominios. Esos terceros pueden recibir datos de conexión cuando el navegador carga sus recursos. Floatly no controla sus prácticas de privacidad.

## 6. Almacenamiento, seguridad y conservación

La configuración, los registros de diagnóstico y los datos de sesión descritos en esta política se almacenan localmente en el perfil de Windows del usuario. Floatly no conserva copias de esos datos en servidores del editor.

La aplicación utiliza los mecanismos normales de almacenamiento y permisos de Windows y WebView2. Floatly no añade una capa propia de cifrado a los archivos locales de configuración y diagnóstico. Por eso, la seguridad de esos archivos también depende de la protección de la cuenta de Windows, del dispositivo y del acceso físico al ordenador.

La seguridad de la conexión depende de la dirección introducida por el usuario. Se recomiendan direcciones HTTPS. Un sitio accedido mediante HTTP no ofrece la misma protección de cifrado en tránsito que una conexión HTTPS.

Los datos locales permanecen en el dispositivo hasta que el usuario los elimine o desinstale la aplicación con la opción de eliminación de datos seleccionada. Los archivos de diagnóstico se rotan automáticamente, con límites de tamaño y de copias anteriores; no existe un plazo de conservación en servidores del editor porque estos registros no se le envían.

## 7. Controles del usuario

El usuario puede controlar los datos y el uso de Floatly de las siguientes maneras:

- **Cambiar el sitio web:** sustituir o eliminar la dirección configurada en los ajustes de la aplicación.
- **Finalizar la sesión del sitio:** cerrar Floatly no garantiza por sí solo la eliminación de cookies o datos de sesión. Estos permanecen en el perfil local hasta que se eliminen.
- **Eliminar solo los datos de navegación de Floatly:** con la aplicación completamente cerrada, eliminar la carpeta %APPDATA%\WidgetWebsite\webview-data. Esto elimina la sesión local de WebView2 y puede exigir volver a iniciar sesión en los sitios. Haz una copia antes si deseas conservar esos datos.
- **Eliminar todos los datos de la aplicación:** al desinstalar, seleccionar la opción del instalador para eliminar los datos de la aplicación. Esta opción elimina las carpetas locales de datos de Floatly, incluidas la configuración, los registros y la sesión de WebView2.
- **Eliminar manualmente los datos:** con la aplicación cerrada, eliminar las carpetas %APPDATA%\WidgetWebsite y, si existe, %LOCALAPPDATA%\WidgetWebsite. Esto elimina la configuración, los diagnósticos y los datos de sesión.
- **Controlar lo que se comparte con los sitios:** elegir cuidadosamente la dirección utilizada, revisar las políticas de los sitios y proveedores de inicio de sesión y abrir los enlaces externos en el navegador predeterminado.

La eliminación de datos locales puede no borrar información que los sitios web o proveedores de autenticación ya hayan recibido y almacenado. Para esa información, el usuario debe ponerse en contacto directamente con esos servicios.

## 8. Derechos de protección de datos

Cuando la legislación aplicable, incluida la Ley General de Protección de Datos de Brasil (LGPD), conceda derechos al titular, este podrá solicitar al responsable de Floatly, según corresponda:

- confirmación de la existencia del tratamiento y acceso a la información;
- corrección de datos personales incompletos, inexactos o desactualizados;
- información sobre el uso y el intercambio de datos;
- anonimización, bloqueo o eliminación de datos tratados en incumplimiento de la legislación, cuando corresponda;
- eliminación de datos tratados sobre la base del consentimiento, con las excepciones legales de conservación;
- revisión de decisiones y los demás derechos previstos por la legislación aplicable.

Las solicitudes sobre el tratamiento realizado por el editor deben enviarse al contacto indicado en la sección 2. Como la mayor parte de los datos de Floatly permanece únicamente en el ordenador del usuario, el editor no puede consultar ni eliminar archivos locales de forma remota. El usuario puede utilizar los controles descritos en la sección 7.

Los datos tratados directamente por sitios web, proveedores de autenticación, Microsoft u otros terceros deben solicitarse a esos responsables respectivos.

## 9. Niños y adolescentes

Floatly es una herramienta general de navegación compacta para Windows, no un servicio de cuentas ni una plataforma diseñada específicamente para niños. La aplicación no solicita intencionadamente información personal de niños. Sin embargo, los sitios web a los que acceda el usuario pueden mostrar contenido o solicitar información propia; esas actividades se rigen por las políticas y controles de los servicios correspondientes y por la legislación aplicable.

## 10. Cambios en esta política

Esta política podrá actualizarse cuando cambien las funciones de Floatly, las prácticas de tratamiento de datos o los requisitos legales y de Microsoft Store. La fecha de actualización se modificará al inicio del documento. Los cambios importantes deberán comunicarse a través de los canales públicos del proyecto y/o de la página de la aplicación, según corresponda.

## 11. Contacto

Para preguntas, solicitudes de privacidad o para ejercer derechos relacionados con el tratamiento de datos realizado por Floatly, ponte en contacto con:

**Responsable:** Pablo Phillipe Cândido dos Santos  
**Correo electrónico:** pablopcsantos@outlook.com
