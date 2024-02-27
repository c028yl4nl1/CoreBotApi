mod banco;
mod config;
mod start;

use banco::*;
use config::*;
use reqwest::blocking::multipart::*;
use reqwest::{blocking::*, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::format;
use std::{io::Split, thread::spawn};
use colored::*;

fn main() {
    let mut http = Client::new();
    let mut id = 0;
    banco::methods::sqlite3::create_banco_sql(config::bancodedados_sql);
    loop {
        match resques_get_updates(token_bot, &http, id as i128 + 1) {
            Ok(mut option) => {
                let option = option.unwrap();
                let json = string_to_json(option);
                if let Some(array_user) = json["result"].as_array() {
                    for user in array_user {
                        id = user["update_id"].as_u64().unwrap_or(0) as i64;
                        let json = user.clone();
                        spawn(move || construtor_msg(json));
                    }
                }

                continue;
            }
            Err(status) => {
                println!("Erro: {:?} Code", status);
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn resques_get_updates<T: AsRef<str>>(
    token: T,
    http: &Client,
    update_last: i128,
) -> Result<Option<String>, reqwest::Error> {
    let info_reqwest = http
        .get(format!(
            "{}bot{}/getUpdates?offset={}",
            api,
            token.as_ref(),
            update_last
        ))
        .send();
    match info_reqwest {
        Ok(reqwest) => {
            if let Ok(x_y) = reqwest.text() {
                return Ok(Some(x_y));
            } else {
                Ok(None)
            }
        }
        Err(reqwest) => Err(reqwest),
    }
}
fn string_to_json<T: AsRef<str> + std::fmt::Debug>(string_response: T) -> Value {
    let serde_: serde_json::Value = serde_json::from_str(string_response.as_ref().into()).unwrap();
    serde_
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Chat {
    first_name: String,
    id: u64,
    #[serde(rename = "type")]
    chat_type: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct From {
    first_name: String,
    id: i64,
    is_bot: bool,
    language_code: String,
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    chat: Chat,
    date: u64,
    from: From,
    message_id: u64,
    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Update {
    message: Message,
    update_id: u64,
}

trait encoder_all{
        fn enconder_url(&self) -> String;
        
}
impl<T: AsRef<str>>  encoder_all for T{
    fn enconder_url(&self) -> String {
        let encoder = encode(self.as_ref()).to_string();
        encoder

    }
}

trait Bot {
    fn enviar_msg<T: AsRef<str>>(&self, msg: &T) -> Option<i64>;
    fn menssage_reply<T: AsRef<str>>(&self, msg: &T) -> Option<i64>;
    fn editar_mgs<T: AsRef<str>>(&self, msg: &T, id: i64);
    fn command(&self) -> Option<Vec<String>>;
    fn enviar_msg_id<T: AsRef<str>>(&self, id: i64, msg: &T) -> bool;
    fn enviar_arquivo<T: AsRef<[u8]> + 'static, P: AsRef<str>>(
        &self,
        array: T,
        filename: P,
        caption: P,
    ) -> bool;

    fn delete_message_id(&self, id: i64) -> bool;
}

impl Bot for Update {
    fn enviar_msg<T: AsRef<str>>(&self, msg: &T) -> Option<i64> {
        let http = Client::new();
        let api_form = format!(
            "{}bot{}/sendMessage?chat_id={}&text={}&parse_mode=html",
            api,
            token_bot,
            self.message.chat.id.to_string(),
            msg.as_ref().enconder_url()
        );
        if let Ok(new) = http.get(api_form).send() {
            let update: Value = serde_json::from_str(new.text().unwrap().as_str()).unwrap();
            let id = &update["result"]["message_id"];
            if id.is_null() {
                return None;
            } else {
                return Some(id.as_i64().unwrap());
            }
        } else {
            None
        }
    }

    fn editar_mgs<T: AsRef<str>>(&self, msg: &T, id: i64) {
        // editMessageText?chat_id={}&message_id={}&text={}
        let http = Client::new();
        let api_form = format!(
            "{}bot{}/editMessageText?chat_id={}&message_id={}&text={}&parse_mode=html
        ",
            api,
            token_bot,
            self.message.chat.id.to_string(),
            id,
            msg.as_ref().enconder_url()
        );
        let http = http.get(api_form).send();
    }
    fn menssage_reply<T: AsRef<str>>(&self, msg: &T) -> Option<i64> {
        let http = Client::new();
        let api_form = format!(
            "{}bot{}/sendMessage?chat_id={}&text={}&reply_to_message_id={}&parse_mode=html",
            api,
            token_bot,
            self.message.chat.id.to_string(),
            msg.as_ref().enconder_url(),
            self.message.message_id
        );
        if let Ok(new) = http.get(api_form).send() {
            let update: Value = serde_json::from_str(new.text().unwrap().as_str()).unwrap();
            let id = &update["result"]["message_id"];
            if id.is_null() {
                return None;
            } else {
                return Some(id.as_i64().unwrap());
            }
        } else {
            None
        }
    }

    fn command(&self) -> Option<Vec<String>> {
        let msg = self.message.text.clone().to_owned();

        if msg.as_bytes()[0] == 0x2f {
            let msg = msg.replace("/", "");
            let msg_split: Vec<String> = msg.split(" ").map(|s| s.to_owned()).collect();
            return Some(msg_split);

            //return Some(vec!["ss","ss"]);
        }
        None
    }

    fn enviar_msg_id<T: AsRef<str>>(&self, id: i64, msg: &T) -> bool {
        let http = Client::new();
        let api_form = format!(
            "{}bot{}/sendMessage?chat_id={}&text={}&parse_mode=html",
            api,
            token_bot,
            id,
            msg.as_ref().enconder_url(),
        );
        let api_get = http.get(api_form).send();

        if let Ok(xu) = api_get{
            let valor =  xu.status().as_u16();
            if valor != 200{
                return false;
            }
            else { return  true; }
        }
        false
    }

    fn enviar_arquivo<T: AsRef<[u8]> + 'static, P: AsRef<str>>(
        &self,
        array: T,
        filename: P,
        caption: P,
    ) -> bool {
        let part = Part::bytes(array.as_ref().to_owned())
            .file_name(filename.as_ref().to_owned())
            .mime_str("application/octet-stream");
        let client = Client::new();
        let api_form = format!("{}bot{}/sendDocument", api, token_bot);
        let form = Form::new()
            .text(
                "chat_id",
                format!("{}", self.message.chat.id).as_str().to_owned(),
            )
            .text("caption", caption.as_ref().to_owned())
            .text("parse_mode", "HTML")
            .text(
                "reply_to_message_id",
                format!("{}", self.message.message_id).as_str().to_owned(),
            )
            .part("document", part.unwrap());
        let x_v_ = client.post(&api_form).multipart(form).send();
        if x_v_.is_err() {
            false
        } else {
            true
        }
    }

    fn delete_message_id(&self, id: i64) -> bool {
        let api_form = format!(
            "{}bot{}/deleteMessage?chat_id={}&message_id={}",
            api, token_bot, self.message.chat.id, id
        );
        let cliente = Client::new().get(api_form).send();
        if cliente.is_ok() {
            true
        } else {
            false
        }
    }


}

fn construtor_msg(msg: Value) -> Result<(), serde_json::Error> {
    let update: Update = serde_json::from_value(msg)?;

    let user = banco::methods::sqlite3::adcionar_usuario_banco_de_dados(
        format!("{}", update.message.chat.id).as_str(),
        &update.message.from.first_name.as_str(),
        bancodedados_sql,
    );

    println!(r"
    \   Usuario: {}  /
    \   Mensagem: {}  /
    \   Pais: {}  /
    \   Username: @{}  /
    ", update.message.from.username.bright_black(),update.message.text.bright_cyan(), update.message.from.language_code.red(), update.message.from.first_name.bright_cyan());
    if user {
        update.enviar_msg_id(
            config::id_dono,
            &format!("Novo usuario: {}", update.message.from.first_name),
        );
    }
    let a = 2;

    commands(update); // Move

    Ok(())
}

fn commands(bot: Update) {
    let command = &bot.command();
    let id = &bot.message.chat.id;
    if let Some(commands) = command {
        let command = commands;
        match command[0].as_str() {
            "start" | "ajuda" | "help" => {
                bot.menssage_reply(
                    &msg_start(
                        &bot.message.from.first_name,
                        bot.message.chat.id.to_owned() as i64,
                    )
                    .to_owned(),
                );
                return ();
            }
            "search" => {
                let id_msg = bot.menssage_reply(&"Search await / espere ").unwrap();

                if command.len() == 1 || !command[1].contains(".") || command[1].contains("'") {
                    bot.editar_mgs(&"Help [/start,/help,/ajuda]", id_msg);
                    return ();
                }
                let info_user = banco::methods::sqlite3::view_user(
                    config::bancodedados_sql,
                    format!("{}", bot.message.chat.id).as_str(),
                )
                .unwrap();
                let keys = (info_user.id, info_user.saldo);
                if keys.1.parse::<i128>().unwrap() >= config::saldo_retirado {
                    let mut String_value = String::new();
                    let url = extrair_dominio(command[1].as_str().clone());
                    if url.is_none() {
                        bot.editar_mgs(&"<b>incorrect url</b>", id_msg);
                        return ();
                    }
                    let resposta = banco::methods::mysql_conector::consult2(url.unwrap()); // Usar o regex para remove o https e o //
                    if let Some(valor) = resposta {
                        for linha in valor {
                            let string_mut = format!(
                                "Host: {}{}\nUsername: {}\nPassword: {}\n\n",
                                linha.url, linha.path, linha.username, linha.password
                            );
                            String_value.push_str(string_mut.as_str());
                        }
                        let send_file = bot.enviar_arquivo(
                            String_value.as_bytes().to_owned(),
                            command[1].as_str(),
                            format!("<b>found: <code>{}</code> ✅ </b>", command[1].as_str()).as_str(),
                        );
                        if send_file {
                            // Remover os pontos
                            let valor = banco::methods::sqlite3::updater_saldo(
                                config::bancodedados_sql,
                                keys.0.as_str(),
                                config::saldo_retirado as i32,
                                "-",
                            );
                        }
                        bot.delete_message_id(id_msg);
                        return ();
                    } else {
                        bot.editar_mgs(&format!("<b><i>Not found / Não encontrado  {}</i></b>", command[1]), id_msg);
                        return ();
                    }
                } else {
                    bot.editar_mgs(&format!("Money account: {}", keys.1), id_msg);
                    return ();
                }
                let resposta = banco::methods::mysql_conector::consult2(command[1].as_str());
                println!("{:?}", resposta);
            }
            "gift" => {
                let send = bot.enviar_msg(&"Checking...").unwrap();
                if command.len() == 1 || command[1].contains("'") {
                    bot.editar_mgs(&"<b>Use :<i> /gift value</i></b>", send);
                    return ();
                }
                let valor = banco::methods::sqlite3::consult_gift_and_adduser(
                    config::bancodedados_sql,
                    command[1].as_str(),
                    format!("{}", bot.message.chat.id).as_str(),
                );

                if let Some(valor_gift) = valor {
                    let insert = true;
                    if insert {
                        bot.editar_mgs(
                            &format!("<b>Gift :<i> Added successfully: {}  </i></b>", valor_gift),
                            send,
                        );
                        return ();
                    } else {
                        bot.editar_mgs(&"<b>Gift :<i>You account no found</i></b>", send);
                        return ();
                    }
                } else {
                    bot.editar_mgs(&"<b>Gift :<i> Ops Gift invalid  </i></b>", send);
                    return ();
                }
                // let info = (bot.message.chat.id, );
            }

            "full" => {
                let send = bot
                    .menssage_reply(&"<blockquote>Creating ...</blockquote>")
                    .unwrap();
                if bot.message.from.id == config::id_dono  {
                    if command.len() == 1
                        || command[1].contains("'")
                        || command[1].parse::<i32>().is_err()
                    {
                        bot.editar_mgs(&"<b>ADM use :<i>/full value</i></b>", send);
                        return ();
                    } else {
                        let valor = command[1].parse::<i64>().unwrap_or(0);
                        if let Some(gift) = banco::methods::sqlite3::create_table_and_gift(
                            config::bancodedados_sql,
                            valor as i32,
                        ) {
                            let format = format!(
                                "<b>Create gift {}: </b> <pre>/gift {} </pre>",
                                valor, gift
                            );
                            bot.editar_mgs(&format, send);
                            return ();
                        }
                    }
                } else {
                    bot.editar_mgs(&"<b>You not  is adm:<i> stupid </i></b>", send);
                }
            }
            "myaccount" => {
                let send = bot
                    .menssage_reply(&"<b>Operation:<i>Analyzing your account</i></b>")
                    .unwrap();
                let info = banco::methods::sqlite3::view_user(
                    config::bancodedados_sql,
                    format!("{}", bot.message.chat.id).as_str(),
                );
                if let Some(info) = info {
                    let format = format!(
                        "<b>Name</b>:<i>{}</i>
<b>Money</b>:<i>{}</i>
<b>Id</b>:<code>{}</code>",
                        info.first_name, info.saldo, info.id
                    );
                    bot.editar_mgs(&format, send);
                } else {
                    bot.editar_mgs(&"Account Not found ", send);
                }
            },

            "admin" => {

                if commands.len() == 1{
                    bot.enviar_msg(&"Escreva uma mensagem admin");
                }
                else {
                    let users = banco::methods::sqlite3::view_user_list(config::bancodedados_sql);
                    match users {

                        Some(valor) => {
                            let msg = bot.menssage_reply(&"Enviando").unwrap();
                            let mut contador: i32 = 0;
                            let mut contador_n_enviado = 0;
                            for user in valor{
                                let msg_ = format!("<b>{}   <i>{}</i></b>", user.first_name , command[1..].join(" "));
                                if bot.enviar_msg_id(user.id.parse::<i64>().unwrap(), &msg_){
                                    contador +=1;
                                }
                                else {
                                    contador_n_enviado +=1;
                                }

                                std::thread::sleep(std::time::Duration::from_millis(100));
                            }
                            bot.editar_mgs(&format!("Recebido: {}, Nao recebido: {} , Total: {} ", contador, contador_n_enviado, (contador_n_enviado + contador)) , msg);
                        }
                        None =>{
                            bot.enviar_msg(&"Sem usuarios");
                            ();
                        }
                    }
                }
            }

            _ => {
                bot.menssage_reply(&"Command not found");
                return ();
            }
        }
    } else {
        //Caso o usuario enviar qualquer coisa que não seja um comando ou um texto
        return ();
    }
}

use regex::Regex;

fn extrair_dominio(url: &str) -> Option<&str> {
    let re = Regex::new(r"https?://([^/ ]+)").unwrap();

    if let Some(cap) = re.captures(url) {
        if let Some(dominio) = cap.get(1) {
            return Some(dominio.as_str());
        }
    } else {
        let re_no_prefix = Regex::new(r"(?:https?:)?([^/ ]+)").unwrap();
        if let Some(cap) = re_no_prefix.captures(url) {
            if let Some(dominio) = cap.get(1) {
                return Some(dominio.as_str());
            }
        }
    }

    None
}
use urlencoding::encode;
fn msg_start<T: AsRef<str>>(name: &T, id: i64) -> String {
    let msg = format!(
        "
<b>
Hello {} Welcome
id: <code> {} </code>

        Commands:

<i>

/myaccount _> view your account
    ver sua conta
/search target view password and user
    procura por senhas
exemples:
    /search login.exemple.com


</i>
🇷🇺
<blockquote>
В запросе должен быть правильный URL-адрес страницы входа, поскольку бот не будет искать поддомены!
Только цель, не указывайте каталог!
За каждый успешный запрос с вашего баланса будет снято 10 баллов. </blockquote>
🇷🇺

🇺🇸
<blockquote>

The query must be the correct URL of the login page as the bot will not search for subdomains !
Just the target, don't put the directory !
Each successful query will deduct 10 points from your balance. </blockquote>
🇺🇸

🇧🇷
<blockquote>
A consulta deve ser a URL correta da página de login, pois o bot não procurará subdomínios !
Apenas o alvo, não coloque o diretório !
Cada consulta bem-sucedida deduzirá 10 pontos do seu saldo. </blockquote>
🇧🇷

Administrator: @Kaiouiue </b> ",
        name.as_ref(),
        id
    );

   msg
}
