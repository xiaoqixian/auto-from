#### Target

​	众所周知，question mark `?` 是 Rust 为数不多的语法糖中最甜的那一颗，其允许你在调用可能抛出异常的函数时，若抛出异常，则继续抛出给自己的调用者。这颗语法糖帮我们省略了很多非必要的错误处理过程。

​	然而，`?` 只在抛出的异常类型 `E1`「可隐式转换」到函数的返回异常类型 `E2` 的情况下有用。可以隐式转换，即意味着 `E2` 实现了 `From<E1>` trait. 倘若我们需要在一个函数中调用可能返回多种异常类型的函数（这种情况经常发生），要么拒绝使用 `?`，要么定义一种类型 `enum Error`，其包含了所有可能抛出的异常类型，同时对这些异常类型实现了 `From` trait，也就是一种万能的异常类型，这样就可以随时使用 question mark 了。

```rust
enum Error {
  IOError(std::io::Error),
  FmtError(std::fmt::Error)
}
impl From<std::io::Error> for Error {
  fn from(err: std::io::Error) -> Self {
    Self::IOError(err)
  }
}
impl From<std::fmt::Error> for Error {
  fn from(err: std::fmt::Error) -> Self {
    Self::FmtError(err)
  }
}
```

显然，为 `Error` 实现 `From` trait 是一项创造性极低的工作。因此我们希望写一个宏为我们自动完成这项工作。

#### 过程宏

​	Rust 的过程宏包括三种：函数宏、继承宏和属性宏。前两者暂时不涉及，只看用途最广的属性宏。

##### 创建一个过程宏项目

​	相比声明宏可以随处定义，过程宏必须定义在一个单独的 crate 中。并在 Cargo.toml 文件中声明

```toml
[lib]
proc-macro = true
```

过程宏的 crate 不能在当前 crate 中直接使用，所以在测试时可以创建一个 main.rs 文件作为 bin target。

​	Rust 的过程宏是一门直接操纵代码流的艺术，编译器将特定的代码流交给你，由你进行处理后再返回给编译器，编译器再根据你的代码流进行编译。以属性宏为例，属性宏本质上是这样一个函数：

```rust
fn (attr: TokenStream, input: TokenStream) -> TokenStream;
```

第一个参数表示用户输入的属性，暂时不看；第二个参数表示当前属性宏所标注的代码体。在过程宏中，任何代码都可以使用 `TokenStream` 类型表示。你可以直接通过 `println!` 在编译期打印 `TokenStream` 的内容：

```rust
//// lib.rs
#[proc_macro_attribute]
pub fn auto_throw(_attr: TokenStream, input: TokenStream) -> TokenStream {
    println!("input: {}", input);
    input
} 

//// main.rs
#[auto_throw]
enum Error {
  IOError(std::io::Error),
  FmtError(std::fmt::Error)
}

fn main() {}
```

编译后就可以看到 `input` 中包含了 `Error` 的完整结构。

​	若在过程宏函数中发生 panic，则会发生编译器报错。

##### syn

​	有了源代码，接下来的内容就是对源代码进行解析获取我们需要的内容，并生成一些新的代码。

​	syn crate 可以帮助我们完成代码解析的过程，这里先不涉及其核心机制，只讲使用方法。syn crate 允许我们对一段代码流做某种预设，并按照预设的结构进行解析。譬如，这里我们预设宏标注的必须是 enum 类型，则可以

```rust
let enm = parse_macro_input!(input as syn::ItemEnum);
```

`as` 语句表示希望按照某种格式进行解析，`ItemEnum` 则是 syn 提供的解析 enum 的类型。看看 `ItemEnum` 的结构

```rust
pub struct ItemEnum {
    pub attrs: Vec<Attribute>, // enum 被标注的属性，例如 #[repr(transparent)]
    pub vis: Visibility, // enum 的可见性
    pub enum_token: Enum, // enum 关键字
    pub ident: Ident, // enum 的名字
    pub generics: Generics, // enum 使用的泛型
    pub brace_token: Brace, // 花括号
    pub variants: Punctuated<Variant, Comma>, // enum 的所有字段，以,分隔
}
```

可以看出，`ItemEnum` 包含了一个 enum 定义中的全部元素，甚至连花括号都有。

​	有了这些类型，我们就可以直接通过 Rust 代码进行读写，从而间接对代码流进行读写。这个过程本质上和 HTTP 协议解析、JSON 文件的反序列化是一致的。

##### quote crate

​	最终，我们的过程宏函数还是需要返回一个 `TokenStream`，我们可以通过 `TokenStream` 本身进行构造，但是那样会很麻烦。quote crate 为我们提供了更方便的工具。

​	quote crate 的核心工具是 quote macro，我们可以直接在 quote macro 中编写 Rust 代码，它会自动转为代码流。最关键的是，你可以通过 `#` 符号引用外部变量完成插值。例如，

```rust
#[proc_macro_attribute]
pub fn auto_throw(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enm = parse_macro_input!(input as syn::ItemEnum);
    TokenStream::from(quote! {
        #enm
    })
}
```

上面的代码将输入代码流解析为 `ItemEnum` 结构体后，又直接使用插值插入回去，所以最终的代码还是原始定义。

​	之所以要使用 `TokenStream::from` 进行一次转换，是因为 `quote!` 生成的是 `proc_macro2::TokenStream`。

​	proc_macro 和 proc_macro2 的区别在于

- proc_macro 更加原始，与 Rust 编译器绑定较深（例如不在标准库中但是不需要导入crate就能直接使用）。只能在 proc macro crate 中才能存在，如果你在一个非过程宏 crate 中调用 `use proc_macro::TokenStream;` 会发生报错，而 proc_macro2 没有这个问题。
- proc_macro2 本质上是一个第三方 crate, 需要在 Cargo.toml 导入才能使用。但是 proc_macro2 更加灵活，所以很多过程宏的库均使用 proc_macro2 作为中间类型，只在宏的返回处转换为 `proc_macro::TokenStream` 类型。

此外，quote macro 还可以使用 `*` 将可迭代的类型展开。例如，若 `tt` 是一组可插值的类型，则 `#(#tt)*` 将这组类型迭代展开，`#(#tt),*` 添加了间隔符 `,`。

##### first step

​	有了 syn 和 quote，我们可以尝试生成代码了。对于一个 enum，我们需要的信息包括 enum 名、字段名、字段中的类型。

​	Rust enum 将字段分为三种类型：Named, Unnamed 和 Unit，具体可以参考[这里](偏偏今天脑壳疼)。

​	接下来的任务就变成：迭代 enum 所有的字段，获取字段的名字和里面的类型，使用 quote macro 输出实现的代码，

```rust
#[proc_macro_attribute]
pub fn auto_throw(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let enm = parse_macro_input!(input as syn::ItemEnum);
    let ItemEnum { variants, ident: enum_name, .. } = enm.clone();
    let impls = variants.into_iter()
        .map(|var| {
            let Variant { fields, ident, .. } = var;
            let fields = match fields {
                Fields::Unnamed(FieldsUnnamed {unnamed,..}) => quote!(#unnamed),
                _ => panic!("Unexpcted field type")
            };
            quote! {
                impl From<#fields> for #enum_name {
                    fn from(item: #fields) -> Self {
                        Self::#ident(item)
                    }
                }
            }
        });    
    TokenStream::from(quote! {
        #enm
        #(#impls)*
    })
}
```

​	可以使用 [cargo expand](https://github.com/dtolnay/cargo-expand) 工具查看生成的代码：

```rust
enum Error {
    IOError(std::io::Error),
    FmtError(std::fmt::Error),
}
impl From<std::io::Error> for Error {
    fn from(item: std::io::Error) -> Self {
        Self::IOError(item)
    }
}
impl From<std::fmt::Error> for Error {
    fn from(item: std::fmt::Error) -> Self {
        Self::FmtError(item)
    }
}
```

在我的[代码](https://github.com/xiaoqixian/auto-from/blob/main/src/main.rs) 还添加了一些对其它类型字段的筛选工作，这里不再赘述。

#### Second Step

​	我们编写的过程宏函数的第一个参数一直没有使用，其表示用户在使用时输入的参数。譬如，我们希望可以给用户指定部分字段不自动实现 `From` 的空间，即

```rust
#[auto_from(disable = [IOError])]
enum Error {
    IOError(std::io::Error),
    FmtError(std::fmt::Error),
}
```

如果此时在 `auto_from` 函数中打印 `attr`，则会得到 `disable = [IOError]` 这样一个 TokenStream。那么问题来了，我们应该如何解析这个 TokenStream 呢？这就涉及到 syn crate 的核心机制 —— the `Parse` trait.

```rust
pub trait Parse: Sized {
    fn parse(input: ParseStream<'_>) -> Result<Self>;
}
```

`Parse` trait 要求类型接受一个 `ParseStream`，并解析出自身，解析过程可能发生错误，但并不直接 panic，而是抛出 error。syn crate 中绝大部分的类型都实现了 `Parse` trait, 实现了 `Parse` trait 可以使用 `parse_macro_input!` 宏从一个 TokenStream 中解析出对应的类型，

```rust
let t = parse_macro_input!(input as T); // where T: Parse
```

​	我们并不需要了解 `ParseStream` 是什么，因为我们并不是一个个字符的进行解析，而是一个一个元素的解析。

​	首先，我们将 `disable = [IOError]` 看作一个 Attribute, 核心元素是属性名、一个作为参数的列表。可以如下定义 Attribute

```rust
pub struct Attribute {
    pub attr_name: Ident,
    pub idents: Vec<Ident>
}
```

在解析过程中，按顺序先解析出名字，等号，然后是列表，所以

```rust
impl Parse for Attribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attr_name = input.parse::<Ident>()?;
        input.parse::<Token![=]>()?;
        let idents;
        let _ = syn::bracketed!(idents in input);
        let idents = Punctuated::<Ident, Token![,]>::parse_terminated(&idents)?;

        Ok(Attribute {
            attr_name,
            idents: idents.into_iter().collect::<Vec<_>>()
        })
    }
}
```

`syn::bracketed!` 允许我们解析出方括号中的一组内容得到一个 `ParseBuffer`。`Punctuated<T, P>` 则可以解析出一组元素类型为 `T`, 间隔符类型为 `P` 的内容，并将元素的内容置于迭代器中。

​	有样学样，所有的 attributes 本质上也是一组逗号隔开的相同元素。所有可以这样定义 `AutoFromAttributes`

```rust
let attrs = Punctuated::<Attribute, Token![,]>::parse_terminated(input)?;
```

如果你认为 `Attribute` 的元素不应全是 `Ident`，则可以将 `Attribute` 改为 enum 类型，然后在 `parse` 中解析不同情况。