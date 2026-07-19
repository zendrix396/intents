import { notFound } from "next/navigation";
import { getAllDocs, getDocBySlug } from "../lib/docs";
import { DocsSidebar } from "../components/DocsSidebar";
import { MdxContent } from "../components/MdxContent";

export function generateStaticParams() {
  const docs = getAllDocs();
  return docs.map((doc) => ({ slug: [doc.slug] }));
}

export async function generateMetadata({ params }: { params: Promise<{ slug?: string[] }> }) {
  const { slug } = await params;
  const docSlug = slug?.[0];
  const doc = docSlug ? getDocBySlug(docSlug) : null;

  if (!doc) return { title: "Docs | Solana Intents" };

  return {
    title: `${doc.meta.title} | Solana Intents`,
    description: doc.meta.description,
  };
}

export default async function DocsPage({ params }: { params: Promise<{ slug?: string[] }> }) {
  const { slug } = await params;
  const docSlug = slug?.[0];
  const docs = getAllDocs();

  // If no slug, show the first doc (getting-started)
  if (!docSlug) {
    const firstDoc = docs[0];
    if (!firstDoc) notFound();

    const doc = getDocBySlug(firstDoc.slug);
    if (!doc) notFound();

    return (
      <div className="flex">
        <DocsSidebar docs={docs} />
        <main className="flex-1 overflow-auto">
          <div className="max-w-3xl mx-auto px-8 py-12">
            <MdxContent source={doc.content} />
          </div>
        </main>
      </div>
    );
  }

  const doc = getDocBySlug(docSlug);
  if (!doc) notFound();

  return (
    <div className="flex">
      <DocsSidebar docs={docs} />
      <main className="flex-1 overflow-auto">
        <div className="max-w-3xl mx-auto px-8 py-12">
          <MdxContent source={doc.content} />
        </div>
      </main>
    </div>
  );
}
