use async_h1::{client, server};
use http_types::Body;
use http_types::Method;
use http_types::Request;
use http_types::Url;
use http_types::{Response, Result};
use pretty_assertions::assert_eq;
mod test_utils;
use test_utils::TestIO;

const BODY: &str = concat![
    "Et provident reprehenderit accusamus dolores et voluptates sed quia. Repellendus odit porro ut et hic molestiae. Sit autem reiciendis animi fugiat deleniti vel iste. Laborum id odio ullam ut impedit dolores. Vel aperiam dolorem voluptatibus dignissimos maxime.",
    "Qui cumque autem debitis consequatur aliquam impedit id nostrum. Placeat error temporibus quos sed vel rerum. Fugit perferendis enim voluptatem rerum vitae dolor distinctio. Quia iusto ex enim voluptatum omnis. Nam et aperiam asperiores nesciunt eos magnam quidem et.",
    "Beatae et sit iure eum voluptatem accusantium quia optio. Tempora et rerum blanditiis repellendus qui est dolorem. Blanditiis deserunt qui dignissimos ad eligendi. Qui quia sequi et. Ipsa error quia quo ducimus et. Asperiores accusantium eius possimus dolore vitae iusto.",
    "Accusantium voluptatum sint dolor iste ut enim laborum quisquam. Iure sunt non quam quasi. Magni officiis necessitatibus omnis consequatur.",
    "Sed modi officia eos explicabo non recusandae praesentium. Est culpa maxime dolorem ullam. In dicta libero aut. Eum voluptatem corporis earum doloribus similique voluptate. Corporis et quia ad ut quia officia.",
    "Porro quod blanditiis molestiae ea. Aut eveniet laborum natus. At repudiandae eos quisquam fugit voluptatibus voluptate. Voluptatibus sint laudantium asperiores eum excepturi autem labore.",
    "Voluptate omnis enim nesciunt tempora. Non eum vero velit velit. Nostrum repudiandae laudantium neque iste minima dicta labore dicta. Velit animi enim ut et tenetur qui et aut. Minus sit eveniet autem repellendus accusamus.",
    "Deleniti qui sit modi quis et ut. Ea ab est tempore adipisci. At voluptas occaecati rem expedita nisi voluptatem iste. Dolor dolorem deleniti hic aliquam. Ullam aspernatur voluptas suscipit corrupti eius fugiat quisquam. Non quaerat dolorem doloremque modi quisquam eaque animi quae.",
    "Voluptas est eaque eaque et quaerat quae dolore. Qui quam et cumque quod. Dolores veritatis dignissimos possimus non. Ipsa excepturi quo autem nemo perferendis. Tempora et repellat accusamus consectetur.",
    "Sint et eum molestiae molestiae explicabo quae. Enim quia repellendus molestias. Harum rerum ut asperiores asperiores. Perferendis officiis iusto ab et ut nulla officia. Qui dicta magni est qui exercitationem et. Quaerat ut commodi beatae iure.",
    "Beatae dolor recusandae dicta vero quibusdam error. Voluptas modi aperiam id. Consequatur id quasi voluptas voluptates doloremque.",
    "Cum explicabo quisquam maiores autem a beatae alias. Corrupti et consequatur repellendus eos rerum iusto. Possimus ipsa totam vero in nam commodi ut eveniet. Facere recusandae commodi tenetur dolor et.",
    "Dolor ut ut architecto incidunt. Sunt tempora quia et similique et. Aut aut rerum soluta quibusdam. Sit deleniti ut veritatis ea nulla eius aut. Quidem doloribus beatae repudiandae ut. Consequatur eveniet consequatur consequatur sunt.",
    "Molestiae debitis et porro quis quas quas quod. Amet beatae placeat qui ut nihil quia. Sunt quos voluptatem id labore. Ut dolorum cupiditate ex velit occaecati velit eaque occaecati. Est ea temporibus expedita ipsum accusantium debitis qui.",
    "Explicabo vitae et maxime in provident natus. Nihil illo itaque eum omnis dolorum eos ratione. Corporis consequuntur nesciunt asperiores tenetur veniam est nulla.",
    "Ut distinctio aut dolor quia aspernatur delectus quia. Molestiae cupiditate corporis fugit asperiores sint eligendi magni. Quo necessitatibus corrupti ea tempore officiis est minus. Nesciunt quos qui minima nostrum nobis qui earum. Temporibus doloremque sed at.",
    "Qui quas occaecati et. Possimus corrupti eaque quis sed accusantium voluptatum ducimus laborum. Alias sapiente et exercitationem ex sequi accusamus ea. Quis id aspernatur soluta et quisquam animi. Aspernatur quasi autem qui. Est dolores iusto perspiciatis.",
    "Itaque incidunt numquam dolores quaerat. Assumenda rerum porro itaque. Ut ratione temporibus occaecati rerum qui commodi.",
    "Nemo nemo iste qui voluptas itaque. Quae quis qui qui cum quod natus itaque est. Dolores voluptate sapiente ipsa eveniet doloremque laboriosam velit sunt. Optio voluptatum doloremque tenetur voluptate.",
    "Recusandae nihil sunt similique minima quis temporibus cum. Laboriosam atque aut tenetur ducimus et vitae. Ducimus qui debitis ut. Non ducimus incidunt optio voluptatum fuga non fugit veritatis. Ut laudantium est minima corporis voluptas inventore qui eum. Rem id aut amet ut.",
    "Et provident reprehenderit accusamus dolores et voluptates sed quia. Repellendus odit porro ut et hic molestiae. Sit autem reiciendis animi fugiat deleniti vel iste. Laborum id odio ullam ut impedit dolores. Vel aperiam dolorem voluptatibus dignissimos maxime.",
    "Qui cumque autem debitis consequatur aliquam impedit id nostrum. Placeat error temporibus quos sed vel rerum. Fugit perferendis enim voluptatem rerum vitae dolor distinctio. Quia iusto ex enim voluptatum omnis. Nam et aperiam asperiores nesciunt eos magnam quidem et.",
    "Beatae et sit iure eum voluptatem accusantium quia optio. Tempora et rerum blanditiis repellendus qui est dolorem. Blanditiis deserunt qui dignissimos ad eligendi. Qui quia sequi et. Ipsa error quia quo ducimus et. Asperiores accusantium eius possimus dolore vitae iusto.",
    "Accusantium voluptatum sint dolor iste ut enim laborum quisquam. Iure sunt non quam quasi. Magni officiis necessitatibus omnis consequatur.",
    "Sed modi officia eos explicabo non recusandae praesentium. Est culpa maxime dolorem ullam. In dicta libero aut. Eum voluptatem corporis earum doloribus similique voluptate. Corporis et quia ad ut quia officia.",
    "Porro quod blanditiis molestiae ea. Aut eveniet laborum natus. At repudiandae eos quisquam fugit voluptatibus voluptate. Voluptatibus sint laudantium asperiores eum excepturi autem labore.",
    "Voluptate omnis enim nesciunt tempora. Non eum vero velit velit. Nostrum repudiandae laudantium neque iste minima dicta labore dicta. Velit animi enim ut et tenetur qui et aut. Minus sit eveniet autem repellendus accusamus.",
    "Deleniti qui sit modi quis et ut. Ea ab est tempore adipisci. At voluptas occaecati rem expedita nisi voluptatem iste. Dolor dolorem deleniti hic aliquam. Ullam aspernatur voluptas suscipit corrupti eius fugiat quisquam. Non quaerat dolorem doloremque modi quisquam eaque animi quae.",
    "Voluptas est eaque eaque et quaerat quae dolore. Qui quam et cumque quod. Dolores veritatis dignissimos possimus non. Ipsa excepturi quo autem nemo perferendis. Tempora et repellat accusamus consectetur.",
    "Sint et eum molestiae molestiae explicabo quae. Enim quia repellendus molestias. Harum rerum ut asperiores asperiores. Perferendis officiis iusto ab et ut nulla officia. Qui dicta magni est qui exercitationem et. Quaerat ut commodi beatae iure.",
    "Beatae dolor recusandae dicta vero quibusdam error. Voluptas modi aperiam id. Consequatur id quasi voluptas voluptates doloremque.",
    "Cum explicabo quisquam maiores autem a beatae alias. Corrupti et consequatur repellendus eos rerum iusto. Possimus ipsa totam vero in nam commodi ut eveniet. Facere recusandae commodi tenetur dolor et.",
    "Explicabo vitae et maxime in provident natus. Nihil illo itaque eum omnis dolorum eos ratione. Corporis consequuntur nesciunt asperiores tenetur veniam est nulla.",
    "Ut distinctio aut dolor quia aspernatur delectus quia. Molestiae cupiditate corporis fugit asperiores sint eligendi magni. Quo necessitatibus corrupti ea tempore officiis est minus. Nesciunt quos qui minima nostrum nobis qui earum. Temporibus doloremque sed at.",
    "Qui quas occaecati et. Possimus corrupti eaque quis sed accusantium voluptatum ducimus laborum. Alias sapiente et exercitationem ex sequi accusamus ea. Quis id aspernatur soluta et quisquam animi. Aspernatur quasi autem qui. Est dolores iusto perspiciatis.",
    "Itaque incidunt numquam dolores quaerat. Assumenda rerum porro itaque. Ut ratione temporibus occaecati rerum qui commodi.",
    "Nemo nemo iste qui voluptas itaque. Quae quis qui qui cum quod natus itaque est. Dolores voluptate sapiente ipsa eveniet doloremque laboriosam velit sunt. Optio voluptatum doloremque tenetur voluptate.",
    "Recusandae nihil sunt similique minima quis temporibus cum. Laboriosam atque aut tenetur ducimus et vitae. Ducimus qui debitis ut. Non ducimus incidunt optio voluptatum fuga non fugit veritatis. Ut laudantium est minima corporis voluptas inventore qui eum. Rem id aut amet ut.",
    "Et provident reprehenderit accusamus dolores et voluptates sed quia. Repellendus odit porro ut et hic molestiae. Sit autem reiciendis animi fugiat deleniti vel iste. Laborum id odio ullam ut impedit dolores. Vel aperiam dolorem voluptatibus dignissimos maxime.",
    "Qui cumque autem debitis consequatur aliquam impedit id nostrum. Placeat error temporibus quos sed vel rerum. Fugit perferendis enim voluptatem rerum vitae dolor distinctio. Quia iusto ex enim voluptatum omnis. Nam et aperiam asperiores nesciunt eos magnam quidem et.",
    "Accusantium voluptatum sint dolor iste ut enim laborum quisquam. Iure sunt non quam quasi. Magni officiis necessitatibus omnis consequatur.",
    "Sed modi officia eos explicabo non recusandae praesentium. Est culpa maxime dolorem ullam. In dicta libero aut. Eum voluptatem corporis earum doloribus similique voluptate. Corporis et quia ad ut quia officia.",
    "Porro quod blanditiis molestiae ea. Aut eveniet laborum natus. At repudiandae eos quisquam fugit voluptatibus voluptate. Voluptatibus sint laudantium asperiores eum excepturi autem labore.",
    "Voluptate omnis enim nesciunt tempora. Non eum vero velit velit. Nostrum repudiandae laudantium neque iste minima dicta labore dicta. Velit animi enim ut et tenetur qui et aut. Minus sit eveniet autem repellendus accusamus.",
    "Deleniti qui sit modi quis et ut. Ea ab est tempore adipisci. At voluptas occaecati rem expedita nisi voluptatem iste. Dolor dolorem deleniti hic aliquam. Ullam aspernatur voluptas suscipit corrupti eius fugiat quisquam. Non quaerat dolorem doloremque modi quisquam eaque animi quae.",
    "Voluptas est eaque eaque et quaerat quae dolore. Qui quam et cumque quod. Dolores veritatis dignissimos possimus non. Ipsa excepturi quo autem nemo perferendis. Tempora et repellat accusamus consectetur.",
    "Sint et eum molestiae molestiae explicabo quae. Enim quia repellendus molestias. Harum rerum ut asperiores asperiores. Perferendis officiis iusto ab et ut nulla officia. Qui dicta magni est qui exercitationem et. Quaerat ut commodi beatae iure.",
    "Beatae dolor recusandae dicta vero quibusdam error. Voluptas modi aperiam id. Consequatur id quasi voluptas voluptates doloremque.",
    "Cum explicabo quisquam maiores autem a beatae alias. Corrupti et consequatur repellendus eos rerum iusto. Possimus ipsa totam vero in nam commodi ut eveniet. Facere recusandae commodi tenetur dolor et.",
    "Dolor ut ut architecto incidunt. Sunt tempora quia et similique et. Aut aut rerum soluta quibusdam. Sit deleniti ut veritatis ea nulla eius aut. Quidem doloribus beatae repudiandae ut. Consequatur eveniet consequatur consequatur sunt.",
    "Molestiae debitis et porro quis quas quas quod. Amet beatae placeat qui ut nihil quia. Sunt quos voluptatem id labore. Ut dolorum cupiditate ex velit occaecati velit eaque occaecati. Est ea temporibus expedita ipsum accusantium debitis qui.",
    "Explicabo vitae et maxime in provident natus. Nihil illo itaque eum omnis dolorum eos ratione. Corporis consequuntur nesciunt asperiores tenetur veniam est nulla.",
    "Ut distinctio aut dolor quia aspernatur delectus quia. Molestiae cupiditate corporis fugit asperiores sint eligendi magni. Quo necessitatibus corrupti ea tempore officiis est minus. Nesciunt quos qui minima nostrum nobis qui earum. Temporibus doloremque sed at.",
    "Qui quas occaecati et. Possimus corrupti eaque quis sed accusantium voluptatum ducimus laborum. Alias sapiente et exercitationem ex sequi accusamus ea. Quis id aspernatur soluta et quisquam animi. Aspernatur quasi autem qui. Est dolores iusto perspiciatis.",
    "Itaque incidunt numquam dolores quaerat. Assumenda rerum porro itaque. Ut ratione temporibus occaecati rerum qui commodi.",
    "Nemo nemo iste qui voluptas itaque. Quae quis qui qui cum quod natus itaque est. Dolores voluptate sapiente ipsa eveniet doloremque laboriosam velit sunt. Optio voluptatum doloremque tenetur voluptate.",
];

#[async_std::test]
async fn server_chunked_large() -> Result<()> {
    let mut request = Request::new(Method::Post, Url::parse("http://domain.com").unwrap());
    //    request.set_body(Body::from_reader(Cursor::new(BODY), None));
    request.set_body(Body::from_string(String::from(BODY)));

    let (mut client, server) = TestIO::new();
    async_std::io::copy(&mut client::Encoder::new(request), &mut client).await?;

    let (request, _) = server::decode(server).await?.unwrap();

    let mut response = Response::new(200);
    response.set_body(Body::from_reader(request, None));

    let response_encoder = server::Encoder::new(response, Method::Get);

    let mut response = client::decode(response_encoder).await?;

    assert_eq!(response.body_string().await?, BODY);
    Ok(())
}
