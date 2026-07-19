import { compileMDX } from "next-mdx-remote/rsc";
import remarkGfm from "remark-gfm";

const components = {};

export async function MdxContent({ source }: { source: string }) {
  const { content } = await compileMDX({
    source,
    components,
    options: {
      mdxOptions: {
        remarkPlugins: [remarkGfm],
      },
    },
  });

  return <>{content}</>;
}
