import { useEffect, useState } from 'react';
import {
  Title1,
  Button,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  Dialog,
  DialogTrigger,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  Label,
  Input,
  Dropdown,
  Option,
  makeStyles,
  Spinner,
  Card,
  shorthands,
  tokens,
  Badge,
} from '@fluentui/react-components';
import { Add24Regular, Delete24Regular } from '@fluentui/react-icons';
import { useProviderStore } from '../stores/providerStore';
import { useI18n } from '../i18n';
import type { ProviderInput } from '../types';

const useStyles = makeStyles({
  toolbar: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: '24px',
  },
  card: {
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    ...shorthands.borderRadius(tokens.borderRadiusXLarge),
    overflow: 'hidden',
  },
  tableRow: {
    ':hover': {
      backgroundColor: tokens.colorNeutralBackground1Hover,
    },
  },
  formField: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    marginBottom: '16px',
  },
  empty: {
    textAlign: 'center',
    padding: '48px',
    color: tokens.colorNeutralForeground3,
  },
});

const PROV_TYPES = ['openai', 'anthropic', 'openai_compatible'];

export default function ProvidersPage() {
  const styles = useStyles();
  const { t } = useI18n();
  const { providers, loading, error, fetchProviders, addProvider, deleteProvider } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState<ProviderInput>({
    name: '',
    prov_type: 'openai',
    base_url: '',
    api_key: '',
    extra_headers: '',
  });

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const handleSubmit = async () => {
    await addProvider(form);
    setOpen(false);
    setForm({ name: '', prov_type: 'openai', base_url: '', api_key: '', extra_headers: '' });
  };

  return (
    <div>
      <div className={styles.toolbar}>
        <Title1>{t.providers.title}</Title1>
        <Dialog open={open} onOpenChange={(_, data: { open: boolean }) => setOpen(data.open)}>
          <DialogTrigger disableButtonEnhancement>
            <Button icon={<Add24Regular />} appearance="primary">{t.providers.addProvider}</Button>
          </DialogTrigger>
          <DialogSurface>
            <DialogBody>
              <DialogTitle>{t.providers.addProvider}</DialogTitle>
              <DialogContent>
                <div className={styles.formField}>
                  <Label>{t.providers.name}</Label>
                  <Input value={form.name} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, name: e.target.value })} placeholder={t.providers.placeholderName} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.providers.type}</Label>
                  <Dropdown value={form.prov_type} onOptionSelect={(_: unknown, data: { optionValue?: string }) => setForm({ ...form, prov_type: data.optionValue || 'openai' })}>
                    {PROV_TYPES.map((t) => (
                      <Option key={t} value={t}>{t}</Option>
                    ))}
                  </Dropdown>
                </div>
                <div className={styles.formField}>
                  <Label>{t.providers.baseUrl}</Label>
                  <Input value={form.base_url} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, base_url: e.target.value })} placeholder={t.providers.placeholderUrl} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.providers.apiKey}</Label>
                  <Input type="password" value={form.api_key} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, api_key: e.target.value })} />
                </div>
                <div className={styles.formField}>
                  <Label>{t.providers.extraHeaders}</Label>
                  <Input value={form.extra_headers || ''} onChange={(e: React.ChangeEvent<HTMLInputElement>) => setForm({ ...form, extra_headers: e.target.value })} placeholder={t.providers.placeholderHeaders} />
                </div>
              </DialogContent>
              <DialogActions>
                <Button appearance="primary" onClick={handleSubmit}>{t.providers.save}</Button>
                <Button onClick={() => setOpen(false)}>{t.providers.cancel}</Button>
              </DialogActions>
            </DialogBody>
          </DialogSurface>
        </Dialog>
      </div>

      {loading && <Spinner label={t.common.loading} />}
      {error && (
        <Badge appearance="filled" color="danger" style={{ marginBottom: '12px', display: 'block' }}>
          {t.common.error}: {error}
        </Badge>
      )}

      <Card className={styles.card}>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHeaderCell>{t.providers.name}</TableHeaderCell>
              <TableHeaderCell>{t.providers.type}</TableHeaderCell>
              <TableHeaderCell>{t.providers.baseUrl}</TableHeaderCell>
              <TableHeaderCell>{t.providers.apiKey}</TableHeaderCell>
              <TableHeaderCell>{t.providers.actions}</TableHeaderCell>
            </TableRow>
          </TableHeader>
          <TableBody>
            {providers.map((p) => (
              <TableRow key={p.id} className={styles.tableRow}>
                <TableCell><b>{p.name}</b></TableCell>
                <TableCell>
                  <Badge appearance="outline" color="brand">{p.prov_type}</Badge>
                </TableCell>
                <TableCell style={{ fontFamily: 'monospace', fontSize: '12px' }}>{p.base_url}</TableCell>
                <TableCell style={{ fontFamily: 'monospace' }}>{p.api_key}</TableCell>
                <TableCell>
                  <Button icon={<Delete24Regular />} appearance="subtle" onClick={() => deleteProvider(p.id)} />
                </TableCell>
              </TableRow>
            ))}
            {providers.length === 0 && (
              <TableRow>
                <TableCell colSpan={5}>
                  <div className={styles.empty}>{t.dashboard.noData}</div>
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </Card>
    </div>
  );
}
