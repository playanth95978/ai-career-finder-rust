import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { EmbeddingStatus } from 'app/entities/enumerations/embedding-status.model';
import { JobSource } from 'app/entities/enumerations/job-source.model';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IJobOffer } from '../job-offer.model';
import { JobOfferService } from '../service/job-offer.service';

import { JobOfferFormGroup, JobOfferFormService } from './job-offer-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-job-offer-update',
  templateUrl: './job-offer-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class JobOfferUpdate implements OnInit {
  readonly isSaving = signal(false);
  jobOffer: IJobOffer | null = null;
  embeddingStatusValues = Object.keys(EmbeddingStatus);
  jobSourceValues = Object.keys(JobSource);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected jobOfferService = inject(JobOfferService);
  protected jobOfferFormService = inject(JobOfferFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: JobOfferFormGroup = this.jobOfferFormService.createJobOfferFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ jobOffer }) => {
      this.jobOffer = jobOffer;
      if (jobOffer) {
        this.updateForm(jobOffer);
      }
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const jobOffer = this.jobOfferFormService.getJobOffer(this.editForm);
    if (jobOffer.id === null) {
      this.subscribeToSaveResponse(this.jobOfferService.create(jobOffer));
    } else {
      this.subscribeToSaveResponse(this.jobOfferService.update(jobOffer));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IJobOffer | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(jobOffer: IJobOffer): void {
    this.jobOffer = jobOffer;
    this.jobOfferFormService.resetForm(this.editForm, jobOffer);
  }
}
